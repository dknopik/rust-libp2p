mod emitter;
#[allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    unused_imports,
    unreachable_pub,
    clippy::all
)]
mod generated;
mod intern;
pub(crate) mod muxer;
mod sink_file;
mod sink_ingest;
mod state;
pub(crate) mod stream;
pub mod transport;
mod wire;

use std::path::PathBuf;

use libp2p_identity::PeerId;

use crate::{
    emitter::{Emitter, DEFAULT_RING_BUFFER_CAPACITY, DEFAULT_SINK_CHANNEL_CAPACITY},
    sink_ingest::IngestSinkConfig,
};

pub use transport::Transport as WiretapTransport;

pub struct WiretapConfig {
    pub ring_buffer_capacity: usize,
    pub sink_channel_capacity: usize,
    pub file_path: Option<PathBuf>,
    pub ingest_addr: Option<String>,
    pub client_name: String,
}

impl Default for WiretapConfig {
    fn default() -> Self {
        Self {
            ring_buffer_capacity: DEFAULT_RING_BUFFER_CAPACITY,
            sink_channel_capacity: DEFAULT_SINK_CHANNEL_CAPACITY,
            file_path: None,
            ingest_addr: None,
            client_name: String::new(),
        }
    }
}

pub fn wrap_transport<T>(
    transport: T,
    config: WiretapConfig,
    peer_id: PeerId,
) -> WiretapTransport<T> {
    let emitter = Emitter::new(config.ring_buffer_capacity);

    if let Some(path) = config.file_path {
        let (tx, rx) = tokio::sync::mpsc::channel(config.sink_channel_capacity);
        emitter.add_sink(tx);
        tokio::spawn(sink_file::run_file_sink(path, rx));
    }

    if let Some(addr) = config.ingest_addr {
        let (tx, rx) = tokio::sync::mpsc::channel(config.sink_channel_capacity);
        emitter.add_sink(tx);
        let boot_id = random_boot_id();
        let started_at_ns = web_time::SystemTime::now()
            .duration_since(web_time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos() as i64)
            .unwrap_or(0);
        let ingest_config = IngestSinkConfig {
            addr,
            peer_id: peer_id.to_bytes(),
            client_name: config.client_name,
            boot_id,
            started_at_ns,
        };
        tokio::spawn(sink_ingest::run_ingest_sink(
            ingest_config,
            emitter.clone(),
            rx,
        ));
    }

    WiretapTransport::new(transport, emitter)
}

fn random_boot_id() -> Vec<u8> {
    let mut id = vec![0u8; 16];
    let ts = web_time::SystemTime::now()
        .duration_since(web_time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    for (i, byte) in id.iter_mut().enumerate() {
        *byte = (ts >> ((i % 16) * 8)) as u8;
    }
    id
}
