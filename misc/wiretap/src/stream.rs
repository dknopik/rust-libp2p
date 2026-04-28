use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{
    io::{IoSlice, IoSliceMut},
    prelude::*,
    ready,
};

use crate::{
    emitter::Emitter,
    generated::wiretap::{
        CloseReason, Direction, StreamChunk,
        mod_Envelope::OneOfpayload,
    },
};

const MAX_DETECT_BUF: usize = 1024;

/// Parses the multistream-select negotiation from raw stream bytes to extract the
/// negotiated protocol name. Watches the initiating direction only: written bytes
/// for outbound streams (we are the dialer), read bytes for inbound streams
/// (remote is the dialer). The protocol is the second varint-length-delimited
/// message after the `/multistream/1.0.0\n` header.
struct ProtocolDetector {
    buf: Vec<u8>,
    watch_out: bool,
    done: bool,
}

impl ProtocolDetector {
    fn new(stream_direction: Direction) -> Self {
        Self {
            buf: Vec::new(),
            watch_out: stream_direction == Direction::DIRECTION_OUT,
            done: false,
        }
    }

    fn feed_read(&mut self, data: &[u8]) -> Option<String> {
        if self.watch_out {
            return None;
        }
        self.do_feed(data)
    }

    fn feed_write(&mut self, data: &[u8]) -> Option<String> {
        if !self.watch_out {
            return None;
        }
        self.do_feed(data)
    }

    fn do_feed(&mut self, data: &[u8]) -> Option<String> {
        if self.done {
            return None;
        }
        self.buf.extend_from_slice(data);
        if self.buf.len() > MAX_DETECT_BUF {
            self.done = true;
            return None;
        }
        let result = self.try_parse();
        if result.is_some() {
            self.done = true;
        }
        result
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn try_parse(&self) -> Option<String> {
        let buf = &self.buf[..];
        // Skip past the first varint-delimited message (multistream header)
        let (len1, rest) = unsigned_varint::decode::usize(buf).ok()?;
        if rest.len() < len1 {
            return None;
        }
        let rest = &rest[len1..];
        // Read the second message (the protocol name)
        let (len2, rest) = unsigned_varint::decode::usize(rest).ok()?;
        if rest.len() < len2 {
            return None;
        }
        let msg = &rest[..len2];
        let proto = msg.strip_suffix(b"\n").unwrap_or(msg);
        String::from_utf8(proto.to_vec()).ok()
    }
}

#[pin_project::pin_project(PinnedDrop)]
pub struct InstrumentedStream<S> {
    #[pin]
    inner: S,
    emitter: Emitter,
    stream_alias: Option<u64>,
    detector: Option<ProtocolDetector>,
}

impl<S> InstrumentedStream<S> {
    pub(crate) fn new(
        inner: S,
        emitter: Emitter,
        stream_alias: u64,
        direction: Direction,
    ) -> Self {
        Self {
            inner,
            emitter,
            stream_alias: Some(stream_alias),
            detector: Some(ProtocolDetector::new(direction)),
        }
    }
}

impl<S: AsyncRead> AsyncRead for InstrumentedStream<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        let num_bytes = ready!(this.inner.poll_read(cx, buf))?;
        if num_bytes > 0
            && let Some(&alias) = this.stream_alias.as_ref()
        {
            if let Some(protocol) =
                this.detector.as_mut().and_then(|d| d.feed_read(&buf[..num_bytes]))
            {
                this.emitter.set_stream_protocol(alias, &protocol);
            }
            if this.detector.as_ref().is_some_and(|d| d.is_done()) {
                *this.detector = None;
            }
            this.emitter.emit(OneOfpayload::stream_chunk(StreamChunk {
                stream_alias: alias,
                direction: Direction::DIRECTION_IN,
                data: buf[..num_bytes].to_vec(),
            }));
        }
        Poll::Ready(Ok(num_bytes))
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        let num_bytes = ready!(this.inner.poll_read_vectored(cx, bufs))?;
        if num_bytes > 0
            && let Some(&alias) = this.stream_alias.as_ref()
        {
            let mut data = Vec::with_capacity(num_bytes);
            let mut remaining = num_bytes;
            for buf in bufs.iter() {
                let take = remaining.min(buf.len());
                data.extend_from_slice(&buf[..take]);
                remaining -= take;
                if remaining == 0 {
                    break;
                }
            }
            if let Some(protocol) = this.detector.as_mut().and_then(|d| d.feed_read(&data)) {
                this.emitter.set_stream_protocol(alias, &protocol);
            }
            if this.detector.as_ref().is_some_and(|d| d.is_done()) {
                *this.detector = None;
            }
            this.emitter.emit(OneOfpayload::stream_chunk(StreamChunk {
                stream_alias: alias,
                direction: Direction::DIRECTION_IN,
                data,
            }));
        }
        Poll::Ready(Ok(num_bytes))
    }
}

impl<S: AsyncWrite> AsyncWrite for InstrumentedStream<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        let num_bytes = ready!(this.inner.poll_write(cx, buf))?;
        if num_bytes > 0
            && let Some(&alias) = this.stream_alias.as_ref()
        {
            if let Some(protocol) =
                this.detector.as_mut().and_then(|d| d.feed_write(&buf[..num_bytes]))
            {
                this.emitter.set_stream_protocol(alias, &protocol);
            }
            if this.detector.as_ref().is_some_and(|d| d.is_done()) {
                *this.detector = None;
            }
            this.emitter.emit(OneOfpayload::stream_chunk(StreamChunk {
                stream_alias: alias,
                direction: Direction::DIRECTION_OUT,
                data: buf[..num_bytes].to_vec(),
            }));
        }
        Poll::Ready(Ok(num_bytes))
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        let num_bytes = ready!(this.inner.poll_write_vectored(cx, bufs))?;
        if num_bytes > 0
            && let Some(&alias) = this.stream_alias.as_ref()
        {
            let mut data = Vec::with_capacity(num_bytes);
            let mut remaining = num_bytes;
            for buf in bufs.iter() {
                let take = remaining.min(buf.len());
                data.extend_from_slice(&buf[..take]);
                remaining -= take;
                if remaining == 0 {
                    break;
                }
            }
            if let Some(protocol) = this.detector.as_mut().and_then(|d| d.feed_write(&data)) {
                this.emitter.set_stream_protocol(alias, &protocol);
            }
            if this.detector.as_ref().is_some_and(|d| d.is_done()) {
                *this.detector = None;
            }
            this.emitter.emit(OneOfpayload::stream_chunk(StreamChunk {
                stream_alias: alias,
                direction: Direction::DIRECTION_OUT,
                data,
            }));
        }
        Poll::Ready(Ok(num_bytes))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.project();
        this.inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.project();
        let result = ready!(this.inner.poll_close(cx));
        if let Some(alias) = this.stream_alias.take() {
            this.emitter
                .close_stream(alias, CloseReason::CLOSE_REASON_CLOSE);
        }
        Poll::Ready(result)
    }
}

#[pin_project::pinned_drop]
impl<S> PinnedDrop for InstrumentedStream<S> {
    fn drop(self: Pin<&mut Self>) {
        let this = self.project();
        if let Some(alias) = this.stream_alias.take() {
            this.emitter
                .close_stream(alias, CloseReason::CLOSE_REASON_RESET);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_ms_msg(msg: &[u8]) -> Vec<u8> {
        let mut len_buf = unsigned_varint::encode::usize_buffer();
        let encoded_len = unsigned_varint::encode::usize(msg.len(), &mut len_buf);
        let mut result = encoded_len.to_vec();
        result.extend_from_slice(msg);
        result
    }

    #[test]
    fn detect_protocol_outbound() {
        let mut detector = ProtocolDetector::new(Direction::DIRECTION_OUT);
        let mut data = encode_ms_msg(b"/multistream/1.0.0\n");
        data.extend_from_slice(&encode_ms_msg(b"/meshsub/1.1.0\n"));
        let result = detector.feed_write(&data);
        assert_eq!(result.as_deref(), Some("/meshsub/1.1.0"));
        assert!(detector.is_done());
    }

    #[test]
    fn detect_protocol_inbound() {
        let mut detector = ProtocolDetector::new(Direction::DIRECTION_IN);
        let mut data = encode_ms_msg(b"/multistream/1.0.0\n");
        data.extend_from_slice(&encode_ms_msg(b"/ipfs/id/1.0.0\n"));
        let result = detector.feed_read(&data);
        assert_eq!(result.as_deref(), Some("/ipfs/id/1.0.0"));
    }

    #[test]
    fn detect_protocol_incremental() {
        let mut detector = ProtocolDetector::new(Direction::DIRECTION_OUT);
        let header = encode_ms_msg(b"/multistream/1.0.0\n");
        let proto = encode_ms_msg(b"/meshsub/1.1.0\n");
        assert!(detector.feed_write(&header).is_none());
        assert!(!detector.is_done());
        let result = detector.feed_write(&proto);
        assert_eq!(result.as_deref(), Some("/meshsub/1.1.0"));
    }

    #[test]
    fn detect_ignores_wrong_direction() {
        let mut detector = ProtocolDetector::new(Direction::DIRECTION_OUT);
        let mut data = encode_ms_msg(b"/multistream/1.0.0\n");
        data.extend_from_slice(&encode_ms_msg(b"/meshsub/1.1.0\n"));
        assert!(detector.feed_read(&data).is_none());
        assert!(!detector.is_done());
    }

    #[test]
    fn detect_overflow_gives_up() {
        let mut detector = ProtocolDetector::new(Direction::DIRECTION_OUT);
        let big_data = vec![0u8; MAX_DETECT_BUF + 1];
        assert!(detector.feed_write(&big_data).is_none());
        assert!(detector.is_done());
    }

    #[test]
    fn detect_byte_at_a_time() {
        let mut detector = ProtocolDetector::new(Direction::DIRECTION_OUT);
        let mut data = encode_ms_msg(b"/multistream/1.0.0\n");
        data.extend_from_slice(&encode_ms_msg(b"/libp2p/circuit/relay/0.2.0/hop\n"));
        let mut found = None;
        for byte in &data {
            if let Some(proto) = detector.feed_write(std::slice::from_ref(byte)) {
                found = Some(proto);
                break;
            }
        }
        assert_eq!(found.as_deref(), Some("/libp2p/circuit/relay/0.2.0/hop"));
    }
}
