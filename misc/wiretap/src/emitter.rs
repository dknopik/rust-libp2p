use std::{
    collections::VecDeque,
    fmt,
    sync::{Arc, Mutex},
};

use libp2p_identity::PeerId;
use tokio::sync::mpsc;

use crate::{
    generated::wiretap::{
        CloseReason, Direction, Envelope,
        mod_Envelope::OneOfpayload,
    },
    intern::StringInterner,
    state::AliasTracker,
};

pub(crate) const DEFAULT_RING_BUFFER_CAPACITY: usize = 65536;
pub(crate) const DEFAULT_SINK_CHANNEL_CAPACITY: usize = 8192;

struct EmitterInner {
    seq: u64,
    ring: VecDeque<Arc<Envelope>>,
    ring_capacity: usize,
    sink_txs: Vec<mpsc::Sender<Arc<Envelope>>>,
    interner: StringInterner,
    state: AliasTracker,
    closed: bool,
}

impl EmitterInner {
    fn emit_payload(&mut self, payload: OneOfpayload) {
        if self.closed {
            return;
        }
        let env = Arc::new(Envelope {
            seq: self.seq,
            observed_at_ns: now_ns(),
            payload,
        });
        self.seq += 1;
        let capacity = self.ring_capacity;
        if self.ring.len() >= capacity {
            self.ring.pop_front();
        }
        self.ring.push_back(Arc::clone(&env));
        for tx in &self.sink_txs {
            let _ = tx.try_send(Arc::clone(&env));
        }
    }
}

#[derive(Clone)]
pub(crate) struct Emitter {
    inner: Arc<Mutex<EmitterInner>>,
}

impl fmt::Debug for Emitter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Emitter").finish_non_exhaustive()
    }
}

pub(crate) enum CatchUp {
    Replay(Vec<Arc<Envelope>>),
    Snapshot(Vec<Envelope>),
}

impl Emitter {
    pub(crate) fn new(ring_capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(EmitterInner {
                seq: 1,
                ring: VecDeque::with_capacity(ring_capacity),
                ring_capacity,
                sink_txs: Vec::new(),
                interner: StringInterner::new(),
                state: AliasTracker::new(),
                closed: false,
            })),
        }
    }

    pub(crate) fn add_sink(&self, tx: mpsc::Sender<Arc<Envelope>>) {
        let mut inner = self.inner.lock().unwrap();
        inner.sink_txs.push(tx);
    }

    pub(crate) fn emit(&self, payload: OneOfpayload) {
        let mut inner = self.inner.lock().unwrap();
        inner.emit_payload(payload);
    }

    pub(crate) fn register_peer(&self, peer_id: &PeerId) -> u64 {
        let mut inner = self.inner.lock().unwrap();
        let (alias, upsert) = inner.state.register_peer(peer_id);
        if let Some(upsert) = upsert {
            inner.emit_payload(OneOfpayload::peer_upsert(upsert));
        }
        alias
    }

    pub(crate) fn register_connection(
        &self,
        peer_alias: u64,
        remote_addr: &str,
        local_addr: &str,
        direction: Direction,
    ) -> u64 {
        let mut inner = self.inner.lock().unwrap();
        let (alias, upsert) = inner.state.register_connection(
            peer_alias,
            remote_addr,
            local_addr,
            direction,
            now_ns(),
        );
        inner.emit_payload(OneOfpayload::connection_upsert(upsert));
        alias
    }

    pub(crate) fn register_stream(&self, conn_alias: u64, direction: Direction) -> u64 {
        let mut inner = self.inner.lock().unwrap();
        let (alias, upsert) =
            inner.state.register_stream(conn_alias, direction, now_ns());
        inner.emit_payload(OneOfpayload::stream_upsert(upsert));
        alias
    }

    pub(crate) fn close_stream(&self, stream_alias: u64, reason: CloseReason) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(closed) = inner.state.close_stream(stream_alias, now_ns(), reason) {
            inner.emit_payload(OneOfpayload::stream_closed(closed));
        }
    }

    pub(crate) fn close_connection(&self, conn_alias: u64) {
        let mut inner = self.inner.lock().unwrap();
        let (conn_closed, stream_closeds) = inner.state.close_connection(conn_alias, now_ns());
        for closed in stream_closeds {
            inner.emit_payload(OneOfpayload::stream_closed(closed));
        }
        if let Some(closed) = conn_closed {
            inner.emit_payload(OneOfpayload::connection_closed(closed));
        }
    }

    pub(crate) fn events_from_seq(&self, last_acked: u64) -> CatchUp {
        let inner = self.inner.lock().unwrap();

        if let Some(oldest) = inner.ring.front()
            && last_acked + 1 >= oldest.seq
        {
            let replay: Vec<_> = inner
                .ring
                .iter()
                .filter(|e| e.seq > last_acked)
                .cloned()
                .collect();
            return CatchUp::Replay(replay);
        }

        let mut snapshot_payloads = Vec::new();
        for def in inner.interner.snapshot() {
            snapshot_payloads.push(OneOfpayload::string_def(def));
        }
        let state_payloads = inner.state.generate_snapshot();

        let mut envelopes = Vec::new();
        if let Some((start, rest)) = state_payloads.split_first() {
            envelopes.push(Envelope {
                seq: 0,
                observed_at_ns: now_ns(),
                payload: start.clone(),
            });
            for payload in &snapshot_payloads {
                envelopes.push(Envelope {
                    seq: 0,
                    observed_at_ns: now_ns(),
                    payload: payload.clone(),
                });
            }
            for payload in rest {
                envelopes.push(Envelope {
                    seq: 0,
                    observed_at_ns: now_ns(),
                    payload: payload.clone(),
                });
            }
        }

        CatchUp::Snapshot(envelopes)
    }

    #[allow(dead_code)]
    pub(crate) fn set_closed(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.closed = true;
    }
}

fn now_ns() -> i64 {
    web_time::SystemTime::now()
        .duration_since(web_time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::wiretap::{SnapshotEnd, SnapshotStart};

    #[test]
    fn seq_monotonically_increases() {
        let emitter = Emitter::new(16);
        let (tx, mut rx) = mpsc::channel(16);
        emitter.add_sink(tx);

        emitter.emit(OneOfpayload::snapshot_start(SnapshotStart {}));
        emitter.emit(OneOfpayload::snapshot_end(SnapshotEnd {}));

        let e1 = rx.try_recv().unwrap();
        let e2 = rx.try_recv().unwrap();
        assert_eq!(e1.seq, 1);
        assert_eq!(e2.seq, 2);
        assert!(e2.seq > e1.seq);
    }

    #[test]
    fn ring_buffer_overflow() {
        let emitter = Emitter::new(4);
        for _ in 0..8 {
            emitter.emit(OneOfpayload::snapshot_start(SnapshotStart {}));
        }
        let inner = emitter.inner.lock().unwrap();
        assert_eq!(inner.ring.len(), 4);
        assert_eq!(inner.ring.front().unwrap().seq, 5);
        assert_eq!(inner.ring.back().unwrap().seq, 8);
    }

    #[test]
    fn closed_emitter_drops() {
        let emitter = Emitter::new(16);
        let (tx, mut rx) = mpsc::channel(16);
        emitter.add_sink(tx);
        emitter.set_closed();
        emitter.emit(OneOfpayload::snapshot_start(SnapshotStart {}));
        assert!(rx.try_recv().is_err());
    }
}
