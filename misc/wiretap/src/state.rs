use std::collections::HashMap;

use libp2p_identity::PeerId;

use crate::generated::wiretap::{
    self, mod_Envelope::OneOfpayload, ConnectionClosed, ConnectionUpsert, Direction, PeerUpsert,
    StreamClosed, StreamUpsert,
};

struct PeerInfo {
    alias: u64,
    peer_id: PeerId,
}

struct ConnectionInfo {
    peer_alias: u64,
    remote_addr: String,
    local_addr: String,
    direction: Direction,
    transport_id: u32,
    security_id: u32,
    muxer_id: u32,
    opened_at_ns: i64,
}

struct StreamInfo {
    conn_alias: u64,
    direction: Direction,
    protocol_id: u32,
    opened_at_ns: i64,
}

pub(crate) struct AliasTracker {
    next_peer_alias: u64,
    next_conn_alias: u64,
    next_stream_alias: u64,
    peers: HashMap<PeerId, PeerInfo>,
    connections: HashMap<u64, ConnectionInfo>,
    streams: HashMap<u64, StreamInfo>,
}

impl AliasTracker {
    pub(crate) fn new() -> Self {
        Self {
            next_peer_alias: 0,
            next_conn_alias: 0,
            next_stream_alias: 0,
            peers: HashMap::new(),
            connections: HashMap::new(),
            streams: HashMap::new(),
        }
    }

    pub(crate) fn register_peer(&mut self, peer_id: &PeerId) -> (u64, Option<PeerUpsert>) {
        if let Some(info) = self.peers.get(peer_id) {
            return (info.alias, None);
        }
        let alias = self.next_peer_alias;
        self.next_peer_alias += 1;
        self.peers.insert(
            *peer_id,
            PeerInfo {
                alias,
                peer_id: *peer_id,
            },
        );
        let upsert = PeerUpsert {
            peer_alias: alias,
            peer_id: peer_id.to_bytes(),
        };
        (alias, Some(upsert))
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn register_connection(
        &mut self,
        peer_alias: u64,
        remote_addr: &str,
        local_addr: &str,
        direction: Direction,
        transport_id: u32,
        security_id: u32,
        muxer_id: u32,
        opened_at_ns: i64,
    ) -> (u64, ConnectionUpsert) {
        let alias = self.next_conn_alias;
        self.next_conn_alias += 1;
        self.connections.insert(
            alias,
            ConnectionInfo {
                peer_alias,
                remote_addr: remote_addr.to_owned(),
                local_addr: local_addr.to_owned(),
                direction,
                transport_id,
                security_id,
                muxer_id,
                opened_at_ns,
            },
        );
        let upsert = ConnectionUpsert {
            conn_alias: alias,
            peer_alias,
            remote_addr: remote_addr.to_owned(),
            local_addr: local_addr.to_owned(),
            direction,
            transport_id,
            security_id,
            muxer_id,
            opened_at_ns,
        };
        (alias, upsert)
    }

    pub(crate) fn register_stream(
        &mut self,
        conn_alias: u64,
        direction: Direction,
        protocol_id: u32,
        opened_at_ns: i64,
    ) -> (u64, StreamUpsert) {
        let alias = self.next_stream_alias;
        self.next_stream_alias += 1;
        self.streams.insert(
            alias,
            StreamInfo {
                conn_alias,
                direction,
                protocol_id,
                opened_at_ns,
            },
        );
        let upsert = StreamUpsert {
            stream_alias: alias,
            conn_alias,
            direction,
            protocol_id,
            opened_at_ns,
        };
        (alias, upsert)
    }

    pub(crate) fn set_stream_protocol(
        &mut self,
        stream_alias: u64,
        protocol_id: u32,
    ) -> Option<StreamUpsert> {
        let info = self.streams.get_mut(&stream_alias)?;
        if info.protocol_id == protocol_id {
            return None;
        }
        info.protocol_id = protocol_id;
        Some(StreamUpsert {
            stream_alias,
            conn_alias: info.conn_alias,
            direction: info.direction,
            protocol_id,
            opened_at_ns: info.opened_at_ns,
        })
    }

    pub(crate) fn close_stream(
        &mut self,
        stream_alias: u64,
        closed_at_ns: i64,
        reason: wiretap::CloseReason,
    ) -> Option<StreamClosed> {
        self.streams.remove(&stream_alias)?;
        Some(StreamClosed {
            stream_alias,
            closed_at_ns,
            reason,
        })
    }

    pub(crate) fn close_connection(
        &mut self,
        conn_alias: u64,
        closed_at_ns: i64,
    ) -> (Option<ConnectionClosed>, Vec<StreamClosed>) {
        let mut stream_closeds = Vec::new();
        let orphaned: Vec<u64> = self
            .streams
            .iter()
            .filter(|(_, info)| info.conn_alias == conn_alias)
            .map(|(&alias, _)| alias)
            .collect();
        for alias in orphaned {
            if let Some(closed) = self.close_stream(
                alias,
                closed_at_ns,
                wiretap::CloseReason::CLOSE_REASON_CONN_CLOSED,
            ) {
                stream_closeds.push(closed);
            }
        }
        let conn_closed = self
            .connections
            .remove(&conn_alias)
            .map(|_| ConnectionClosed {
                conn_alias,
                closed_at_ns,
            });
        (conn_closed, stream_closeds)
    }

    pub(crate) fn generate_snapshot(&self) -> Vec<OneOfpayload> {
        let mut payloads = Vec::new();

        payloads.push(OneOfpayload::snapshot_start(wiretap::SnapshotStart {}));

        let mut peers: Vec<_> = self.peers.values().collect();
        peers.sort_by_key(|p| p.alias);
        for p in peers {
            payloads.push(OneOfpayload::peer_upsert(PeerUpsert {
                peer_alias: p.alias,
                peer_id: p.peer_id.to_bytes(),
            }));
        }

        let mut conns: Vec<_> = self.connections.iter().collect();
        conns.sort_by_key(|(alias, _)| *alias);
        for (alias, info) in conns {
            payloads.push(OneOfpayload::connection_upsert(ConnectionUpsert {
                conn_alias: *alias,
                peer_alias: info.peer_alias,
                remote_addr: info.remote_addr.clone(),
                local_addr: info.local_addr.clone(),
                direction: info.direction,
                transport_id: info.transport_id,
                security_id: info.security_id,
                muxer_id: info.muxer_id,
                opened_at_ns: info.opened_at_ns,
            }));
        }

        let mut streams: Vec<_> = self.streams.iter().collect();
        streams.sort_by_key(|(alias, _)| *alias);
        for (alias, info) in streams {
            payloads.push(OneOfpayload::stream_upsert(StreamUpsert {
                stream_alias: *alias,
                conn_alias: info.conn_alias,
                direction: info.direction,
                protocol_id: info.protocol_id,
                opened_at_ns: info.opened_at_ns,
            }));
        }

        payloads.push(OneOfpayload::snapshot_end(wiretap::SnapshotEnd {}));

        payloads
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peer_registration_idempotent() {
        let mut tracker = AliasTracker::new();
        let peer = PeerId::random();
        let (alias1, upsert1) = tracker.register_peer(&peer);
        let (alias2, upsert2) = tracker.register_peer(&peer);
        assert_eq!(alias1, alias2);
        assert!(upsert1.is_some());
        assert!(upsert2.is_none());
    }

    #[test]
    fn close_connection_closes_orphan_streams() {
        let mut tracker = AliasTracker::new();
        let peer = PeerId::random();
        let (peer_alias, _) = tracker.register_peer(&peer);
        let (conn_alias, _) = tracker.register_connection(
            peer_alias,
            "/ip4/1.2.3.4/tcp/1234",
            "/ip4/0.0.0.0/tcp/5678",
            Direction::DIRECTION_OUT,
            0,
            0,
            0,
            100,
        );
        tracker.register_stream(conn_alias, Direction::DIRECTION_OUT, 0, 200);
        tracker.register_stream(conn_alias, Direction::DIRECTION_IN, 0, 300);

        let (conn_closed, stream_closeds) = tracker.close_connection(conn_alias, 400);
        assert!(conn_closed.is_some());
        assert_eq!(stream_closeds.len(), 2);
    }

    #[test]
    fn snapshot_includes_all_entities() {
        let mut tracker = AliasTracker::new();
        let peer = PeerId::random();
        let (peer_alias, _) = tracker.register_peer(&peer);
        let (conn_alias, _) = tracker.register_connection(
            peer_alias,
            "/ip4/1.2.3.4/tcp/1234",
            "/ip4/0.0.0.0/tcp/5678",
            Direction::DIRECTION_OUT,
            0,
            0,
            0,
            100,
        );
        tracker.register_stream(conn_alias, Direction::DIRECTION_OUT, 0, 200);

        let snap = tracker.generate_snapshot();
        // SnapshotStart + PeerUpsert + ConnectionUpsert + StreamUpsert + SnapshotEnd
        assert_eq!(snap.len(), 5);
        assert!(matches!(snap[0], OneOfpayload::snapshot_start(_)));
        assert!(matches!(snap[1], OneOfpayload::peer_upsert(_)));
        assert!(matches!(snap[2], OneOfpayload::connection_upsert(_)));
        assert!(matches!(snap[3], OneOfpayload::stream_upsert(_)));
        assert!(matches!(snap[4], OneOfpayload::snapshot_end(_)));
    }
}
