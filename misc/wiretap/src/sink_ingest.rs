use std::sync::Arc;

use tokio::{
    io::{AsyncWriteExt, BufReader, BufWriter},
    net::TcpStream,
    sync::mpsc,
    time::{Duration, sleep},
};

use crate::{
    emitter::{CatchUp, Emitter},
    generated::wiretap::{ClientHello, Envelope, ServerHello},
    wire::{self, PROTOCOL_VERSION, TYPE_CLIENT_HELLO, TYPE_SERVER_HELLO},
};

pub(crate) struct IngestSinkConfig {
    pub(crate) addr: String,
    pub(crate) peer_id: Vec<u8>,
    pub(crate) client_name: String,
    pub(crate) boot_id: Vec<u8>,
    pub(crate) started_at_ns: i64,
}

pub(crate) async fn run_ingest_sink(
    config: IngestSinkConfig,
    emitter: Emitter,
    mut rx: mpsc::Receiver<Arc<Envelope>>,
) {
    let mut backoff = Duration::from_millis(250);
    let max_backoff = Duration::from_secs(5);

    loop {
        match connect_and_stream(&config, &emitter, &mut rx).await {
            Ok(()) => return,
            Err(e) => {
                tracing::warn!(%e, addr = %config.addr, "wiretap ingest connection lost, reconnecting");
                sleep(backoff).await;
                backoff = (backoff * 2).min(max_backoff);
            }
        }
    }
}

async fn connect_and_stream(
    config: &IngestSinkConfig,
    emitter: &Emitter,
    rx: &mut mpsc::Receiver<Arc<Envelope>>,
) -> std::io::Result<()> {
    let stream = if config.addr.contains('/') {
        #[cfg(unix)]
        {
            use tokio::net::UnixStream;
            let unix = UnixStream::connect(&config.addr).await?;
            TokioStream::Unix(unix)
        }
        #[cfg(not(unix))]
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "unix sockets not supported on this platform",
            ));
        }
    } else {
        let tcp = TcpStream::connect(&config.addr).await?;
        tcp.set_nodelay(true)?;
        TokioStream::Tcp(tcp)
    };

    let (reader, writer) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    let hello = ClientHello {
        protocol_version: PROTOCOL_VERSION,
        peer_id: config.peer_id.clone(),
        client_name: config.client_name.clone(),
        boot_id: config.boot_id.clone(),
        started_at_ns: config.started_at_ns,
    };
    wire::write_typed(&mut writer, TYPE_CLIENT_HELLO, &hello).await?;
    writer.flush().await?;

    let server_hello: ServerHello = wire::read_typed(&mut reader, TYPE_SERVER_HELLO).await?;

    if server_hello.last_acked_seq > 0 {
        let catchup = emitter.events_from_seq(server_hello.last_acked_seq);
        match catchup {
            CatchUp::Replay(envs) => {
                for env in &envs {
                    wire::write_typed(&mut writer, wire::TYPE_ENVELOPE, env.as_ref()).await?;
                }
            }
            CatchUp::Snapshot(envs) => {
                for env in &envs {
                    wire::write_typed(&mut writer, wire::TYPE_ENVELOPE, env).await?;
                }
            }
        }
        writer.flush().await?;
    }

    while let Some(env) = rx.recv().await {
        wire::write_typed(&mut writer, wire::TYPE_ENVELOPE, env.as_ref()).await?;
        if rx.is_empty() {
            writer.flush().await?;
        }
    }

    writer.flush().await?;
    Ok(())
}

enum TokioStream {
    Tcp(TcpStream),
    #[cfg(unix)]
    Unix(tokio::net::UnixStream),
}

impl tokio::io::AsyncRead for TokioStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            TokioStream::Tcp(s) => std::pin::Pin::new(s).poll_read(cx, buf),
            #[cfg(unix)]
            TokioStream::Unix(s) => std::pin::Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl tokio::io::AsyncWrite for TokioStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match self.get_mut() {
            TokioStream::Tcp(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            #[cfg(unix)]
            TokioStream::Unix(s) => std::pin::Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            TokioStream::Tcp(s) => std::pin::Pin::new(s).poll_flush(cx),
            #[cfg(unix)]
            TokioStream::Unix(s) => std::pin::Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            TokioStream::Tcp(s) => std::pin::Pin::new(s).poll_shutdown(cx),
            #[cfg(unix)]
            TokioStream::Unix(s) => std::pin::Pin::new(s).poll_shutdown(cx),
        }
    }
}
