use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::future::{MapOk, TryFutureExt};
use libp2p_core::{
    Multiaddr,
    muxing::StreamMuxer,
    transport::{DialOpts, ListenerId, TransportError, TransportEvent},
};
use libp2p_identity::PeerId;

use crate::{emitter::Emitter, generated::wiretap::Direction, muxer::Muxer};

#[derive(Debug, Clone)]
#[pin_project::pin_project]
pub struct Transport<T> {
    #[pin]
    transport: T,
    emitter: Emitter,
}

impl<T> Transport<T> {
    pub(crate) fn new(transport: T, emitter: Emitter) -> Self {
        Self { transport, emitter }
    }
}

impl<T, M> libp2p_core::Transport for Transport<T>
where
    T: libp2p_core::Transport<Output = (PeerId, M)>,
    M: StreamMuxer + Send + 'static,
    M::Substream: Send + 'static,
    M::Error: Send + Sync + 'static,
{
    type Output = (PeerId, Muxer<M>);
    type Error = T::Error;
    type ListenerUpgrade =
        MapOk<T::ListenerUpgrade, Box<dyn FnOnce((PeerId, M)) -> (PeerId, Muxer<M>) + Send>>;
    type Dial = MapOk<T::Dial, Box<dyn FnOnce((PeerId, M)) -> (PeerId, Muxer<M>) + Send>>;

    fn listen_on(
        &mut self,
        id: ListenerId,
        addr: Multiaddr,
    ) -> Result<(), TransportError<Self::Error>> {
        self.transport.listen_on(id, addr)
    }

    fn remove_listener(&mut self, id: ListenerId) -> bool {
        self.transport.remove_listener(id)
    }

    fn dial(
        &mut self,
        addr: Multiaddr,
        dial_opts: DialOpts,
    ) -> Result<Self::Dial, TransportError<Self::Error>> {
        let emitter = self.emitter.clone();
        let remote_addr = addr.to_string();
        Ok(self
            .transport
            .dial(addr, dial_opts)?
            .map_ok(Box::new(move |(peer_id, stream_muxer)| {
                let peer_alias = emitter.register_peer(&peer_id);
                let conn_alias = emitter.register_connection(
                    peer_alias,
                    &remote_addr,
                    "",
                    Direction::DIRECTION_OUT,
                );
                (peer_id, Muxer::new(stream_muxer, emitter, conn_alias))
            })))
    }

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<TransportEvent<Self::ListenerUpgrade, Self::Error>> {
        let this = self.project();
        match this.transport.poll(cx) {
            Poll::Ready(TransportEvent::Incoming {
                listener_id,
                upgrade,
                local_addr,
                send_back_addr,
            }) => {
                let emitter = this.emitter.clone();
                let remote_str = send_back_addr.to_string();
                let local_str = local_addr.to_string();
                Poll::Ready(TransportEvent::Incoming {
                    listener_id,
                    upgrade: upgrade.map_ok(Box::new(move |(peer_id, stream_muxer)| {
                        let peer_alias = emitter.register_peer(&peer_id);
                        let conn_alias = emitter.register_connection(
                            peer_alias,
                            &remote_str,
                            &local_str,
                            Direction::DIRECTION_IN,
                        );
                        (peer_id, Muxer::new(stream_muxer, emitter, conn_alias))
                    })),
                    local_addr,
                    send_back_addr,
                })
            }
            Poll::Ready(other) => {
                let mapped = other.map_upgrade(|_upgrade| unreachable!("case already matched"));
                Poll::Ready(mapped)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
