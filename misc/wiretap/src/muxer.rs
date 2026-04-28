use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::ready;
use libp2p_core::muxing::{StreamMuxer, StreamMuxerEvent};

use crate::{emitter::Emitter, generated::wiretap::Direction, stream::InstrumentedStream};

#[pin_project::pin_project(PinnedDrop)]
pub struct Muxer<M> {
    #[pin]
    inner: M,
    emitter: Emitter,
    conn_alias: Option<u64>,
}

impl<M> Muxer<M> {
    pub(crate) fn new(inner: M, emitter: Emitter, conn_alias: u64) -> Self {
        Self {
            inner,
            emitter,
            conn_alias: Some(conn_alias),
        }
    }
}

impl<M> StreamMuxer for Muxer<M>
where
    M: StreamMuxer,
{
    type Substream = InstrumentedStream<M::Substream>;
    type Error = M::Error;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<StreamMuxerEvent, Self::Error>> {
        let this = self.project();
        this.inner.poll(cx)
    }

    fn poll_inbound(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Substream, Self::Error>> {
        let this = self.project();
        let inner = ready!(this.inner.poll_inbound(cx)?);
        let conn_alias = this.conn_alias.expect("muxer used after close");
        let stream_alias = this
            .emitter
            .register_stream(conn_alias, Direction::DIRECTION_IN, "");
        Poll::Ready(Ok(InstrumentedStream::new(
            inner,
            this.emitter.clone(),
            stream_alias,
            Direction::DIRECTION_IN,
        )))
    }

    fn poll_outbound(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Substream, Self::Error>> {
        let this = self.project();
        let inner = ready!(this.inner.poll_outbound(cx)?);
        let conn_alias = this.conn_alias.expect("muxer used after close");
        let stream_alias = this
            .emitter
            .register_stream(conn_alias, Direction::DIRECTION_OUT, "");
        Poll::Ready(Ok(InstrumentedStream::new(
            inner,
            this.emitter.clone(),
            stream_alias,
            Direction::DIRECTION_OUT,
        )))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let result = ready!(this.inner.poll_close(cx));
        if let Some(alias) = this.conn_alias.take() {
            this.emitter.close_connection(alias);
        }
        Poll::Ready(result)
    }
}

#[pin_project::pinned_drop]
impl<M> PinnedDrop for Muxer<M> {
    fn drop(self: Pin<&mut Self>) {
        let this = self.project();
        if let Some(alias) = this.conn_alias.take() {
            this.emitter.close_connection(alias);
        }
    }
}
