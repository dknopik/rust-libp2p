use std::io;

use quick_protobuf::{BytesReader, MessageRead, MessageWrite, Writer};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub(crate) const TYPE_CLIENT_HELLO: u8 = 0x01;
pub(crate) const TYPE_SERVER_HELLO: u8 = 0x02;
pub(crate) const TYPE_ENVELOPE: u8 = 0x03;

pub(crate) const MAX_MESSAGE_SIZE: u64 = 4 << 20; // 4 MiB
pub(crate) const PROTOCOL_VERSION: u32 = 3;

pub(crate) fn encode_typed(type_byte: u8, msg: &impl MessageWrite) -> Vec<u8> {
    let size = msg.get_size();
    let mut varint_buf = unsigned_varint::encode::usize_buffer();
    let encoded = unsigned_varint::encode::usize(size, &mut varint_buf);
    let mut buf = Vec::with_capacity(1 + encoded.len() + size);
    buf.push(type_byte);
    buf.extend_from_slice(encoded);
    let mut writer = Writer::new(&mut buf);
    msg.write_message(&mut writer)
        .expect("writing to Vec cannot fail");
    buf
}

pub(crate) async fn read_typed<'a, T, R>(reader: &mut R, expected_type: u8) -> io::Result<T>
where
    T: MessageRead<'a> + Default,
    R: AsyncRead + Unpin,
{
    let mut type_buf = [0u8; 1];
    reader.read_exact(&mut type_buf).await?;
    if type_buf[0] != expected_type {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unexpected message type: got 0x{:02x}, want 0x{:02x}",
                type_buf[0], expected_type
            ),
        ));
    }

    let length = read_varint(reader).await?;
    if length > MAX_MESSAGE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("message too large: {length} bytes"),
        ));
    }

    let mut data = vec![0u8; length as usize];
    reader.read_exact(&mut data).await?;

    let mut br = BytesReader::from_bytes(&data);
    // SAFETY: T does not borrow from data (we use --dont_use_cow so all fields are owned).
    let msg = unsafe {
        let data_ref: &'a [u8] = &*(data.as_slice() as *const [u8]);
        T::from_reader(&mut br, data_ref)
    };
    msg.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

pub(crate) async fn write_typed<W>(writer: &mut W, type_byte: u8, msg: &impl MessageWrite) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let buf = encode_typed(type_byte, msg);
    writer.write_all(&buf).await
}

async fn read_varint<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<u64> {
    let mut value: u64 = 0;
    let mut shift: u32 = 0;
    loop {
        let mut b = [0u8; 1];
        reader.read_exact(&mut b).await?;
        value |= u64::from(b[0] & 0x7f) << shift;
        if b[0] & 0x80 == 0 {
            return Ok(value);
        }
        shift += 7;
        if shift >= 64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "varint overflow",
            ));
        }
    }
}

#[allow(dead_code)]
pub(crate) fn encode_envelope_frame(msg: &impl MessageWrite) -> Vec<u8> {
    encode_typed(TYPE_ENVELOPE, msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::wiretap;

    #[tokio::test]
    async fn round_trip_client_hello() {
        let hello = wiretap::ClientHello {
            protocol_version: PROTOCOL_VERSION,
            peer_id: vec![1, 2, 3],
            client_name: "test".to_owned(),
            boot_id: vec![4, 5, 6],
            started_at_ns: 1234567890,
        };

        let buf = encode_typed(TYPE_CLIENT_HELLO, &hello);
        let mut cursor = io::Cursor::new(buf);
        let decoded: wiretap::ClientHello = read_typed(&mut cursor, TYPE_CLIENT_HELLO).await.unwrap();
        assert_eq!(decoded, hello);
    }

    #[tokio::test]
    async fn round_trip_envelope() {
        let env = wiretap::Envelope {
            seq: 42,
            observed_at_ns: 999,
            payload: wiretap::mod_Envelope::OneOfpayload::stream_chunk(wiretap::StreamChunk {
                stream_alias: 7,
                direction: wiretap::Direction::DIRECTION_OUT,
                data: vec![0xde, 0xad],
            }),
        };

        let buf = encode_typed(TYPE_ENVELOPE, &env);
        let mut cursor = io::Cursor::new(buf);
        let decoded: wiretap::Envelope = read_typed(&mut cursor, TYPE_ENVELOPE).await.unwrap();
        assert_eq!(decoded, env);
    }

    #[tokio::test]
    async fn wrong_type_rejected() {
        let hello = wiretap::ClientHello::default();
        let buf = encode_typed(TYPE_CLIENT_HELLO, &hello);
        let mut cursor = io::Cursor::new(buf);
        let result: io::Result<wiretap::ServerHello> =
            read_typed(&mut cursor, TYPE_SERVER_HELLO).await;
        assert!(result.is_err());
    }
}
