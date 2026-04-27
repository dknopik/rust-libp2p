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

#[pin_project::pin_project(PinnedDrop)]
pub struct InstrumentedStream<S> {
    #[pin]
    inner: S,
    emitter: Emitter,
    stream_alias: Option<u64>,
}

impl<S> InstrumentedStream<S> {
    pub(crate) fn new(inner: S, emitter: Emitter, stream_alias: u64) -> Self {
        Self {
            inner,
            emitter,
            stream_alias: Some(stream_alias),
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
