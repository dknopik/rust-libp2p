use std::{path::PathBuf, sync::Arc};

use quick_protobuf::MessageWrite;
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
    sync::mpsc,
};

use crate::generated::wiretap::Envelope;

pub(crate) async fn run_file_sink(
    path: PathBuf,
    mut rx: mpsc::Receiver<Arc<Envelope>>,
) {
    let file = match File::create(&path).await {
        Ok(f) => f,
        Err(e) => {
            tracing::error!(%e, path = %path.display(), "failed to create wiretap trace file");
            return;
        }
    };
    let mut writer = BufWriter::new(file);

    while let Some(env) = rx.recv().await {
        if let Err(e) = write_length_delimited(&mut writer, &env).await {
            tracing::warn!(%e, "wiretap file sink write error");
            break;
        }
    }

    let _ = writer.flush().await;
}

async fn write_length_delimited(
    writer: &mut BufWriter<File>,
    msg: &Envelope,
) -> std::io::Result<()> {
    let size = msg.get_size();
    let mut varint_buf = unsigned_varint::encode::usize_buffer();
    let encoded = unsigned_varint::encode::usize(size, &mut varint_buf);
    writer.write_all(encoded).await?;

    let mut buf = Vec::with_capacity(size);
    let mut qw = quick_protobuf::Writer::new(&mut buf);
    msg.write_message(&mut qw)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    writer.write_all(&buf).await?;
    Ok(())
}
