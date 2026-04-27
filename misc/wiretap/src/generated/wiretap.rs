// Automatically generated rust module for 'wiretap.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    DIRECTION_UNKNOWN = 0,
    DIRECTION_IN = 1,
    DIRECTION_OUT = 2,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::DIRECTION_UNKNOWN
    }
}

impl From<i32> for Direction {
    fn from(i: i32) -> Self {
        match i {
            0 => Direction::DIRECTION_UNKNOWN,
            1 => Direction::DIRECTION_IN,
            2 => Direction::DIRECTION_OUT,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for Direction {
    fn from(s: &'a str) -> Self {
        match s {
            "DIRECTION_UNKNOWN" => Direction::DIRECTION_UNKNOWN,
            "DIRECTION_IN" => Direction::DIRECTION_IN,
            "DIRECTION_OUT" => Direction::DIRECTION_OUT,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CloseReason {
    CLOSE_REASON_UNKNOWN = 0,
    CLOSE_REASON_CLOSE = 1,
    CLOSE_REASON_RESET = 2,
    CLOSE_REASON_CONN_CLOSED = 3,
}

impl Default for CloseReason {
    fn default() -> Self {
        CloseReason::CLOSE_REASON_UNKNOWN
    }
}

impl From<i32> for CloseReason {
    fn from(i: i32) -> Self {
        match i {
            0 => CloseReason::CLOSE_REASON_UNKNOWN,
            1 => CloseReason::CLOSE_REASON_CLOSE,
            2 => CloseReason::CLOSE_REASON_RESET,
            3 => CloseReason::CLOSE_REASON_CONN_CLOSED,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for CloseReason {
    fn from(s: &'a str) -> Self {
        match s {
            "CLOSE_REASON_UNKNOWN" => CloseReason::CLOSE_REASON_UNKNOWN,
            "CLOSE_REASON_CLOSE" => CloseReason::CLOSE_REASON_CLOSE,
            "CLOSE_REASON_RESET" => CloseReason::CLOSE_REASON_RESET,
            "CLOSE_REASON_CONN_CLOSED" => CloseReason::CLOSE_REASON_CONN_CLOSED,
            _ => Self::default(),
        }
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Envelope {
    pub seq: u64,
    pub observed_at_ns: i64,
    pub payload: wiretap::mod_Envelope::OneOfpayload,
}

impl<'a> MessageRead<'a> for Envelope {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.seq = r.read_uint64(bytes)?,
                Ok(16) => msg.observed_at_ns = r.read_int64(bytes)?,
                Ok(90) => msg.payload = wiretap::mod_Envelope::OneOfpayload::snapshot_start(r.read_message::<wiretap::SnapshotStart>(bytes)?),
                Ok(98) => msg.payload = wiretap::mod_Envelope::OneOfpayload::snapshot_end(r.read_message::<wiretap::SnapshotEnd>(bytes)?),
                Ok(162) => msg.payload = wiretap::mod_Envelope::OneOfpayload::string_def(r.read_message::<wiretap::StringDef>(bytes)?),
                Ok(170) => msg.payload = wiretap::mod_Envelope::OneOfpayload::peer_upsert(r.read_message::<wiretap::PeerUpsert>(bytes)?),
                Ok(178) => msg.payload = wiretap::mod_Envelope::OneOfpayload::connection_upsert(r.read_message::<wiretap::ConnectionUpsert>(bytes)?),
                Ok(186) => msg.payload = wiretap::mod_Envelope::OneOfpayload::connection_closed(r.read_message::<wiretap::ConnectionClosed>(bytes)?),
                Ok(194) => msg.payload = wiretap::mod_Envelope::OneOfpayload::stream_upsert(r.read_message::<wiretap::StreamUpsert>(bytes)?),
                Ok(202) => msg.payload = wiretap::mod_Envelope::OneOfpayload::stream_closed(r.read_message::<wiretap::StreamClosed>(bytes)?),
                Ok(210) => msg.payload = wiretap::mod_Envelope::OneOfpayload::stream_chunk(r.read_message::<wiretap::StreamChunk>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for Envelope {
    fn get_size(&self) -> usize {
        0
        + if self.seq == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.seq) as u64) }
        + if self.observed_at_ns == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.observed_at_ns) as u64) }
        + match self.payload {
            wiretap::mod_Envelope::OneOfpayload::snapshot_start(ref m) => 1 + sizeof_len((m).get_size()),
            wiretap::mod_Envelope::OneOfpayload::snapshot_end(ref m) => 1 + sizeof_len((m).get_size()),
            wiretap::mod_Envelope::OneOfpayload::string_def(ref m) => 2 + sizeof_len((m).get_size()),
            wiretap::mod_Envelope::OneOfpayload::peer_upsert(ref m) => 2 + sizeof_len((m).get_size()),
            wiretap::mod_Envelope::OneOfpayload::connection_upsert(ref m) => 2 + sizeof_len((m).get_size()),
            wiretap::mod_Envelope::OneOfpayload::connection_closed(ref m) => 2 + sizeof_len((m).get_size()),
            wiretap::mod_Envelope::OneOfpayload::stream_upsert(ref m) => 2 + sizeof_len((m).get_size()),
            wiretap::mod_Envelope::OneOfpayload::stream_closed(ref m) => 2 + sizeof_len((m).get_size()),
            wiretap::mod_Envelope::OneOfpayload::stream_chunk(ref m) => 2 + sizeof_len((m).get_size()),
            wiretap::mod_Envelope::OneOfpayload::None => 0,
    }    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.seq != 0u64 { w.write_with_tag(8, |w| w.write_uint64(*&self.seq))?; }
        if self.observed_at_ns != 0i64 { w.write_with_tag(16, |w| w.write_int64(*&self.observed_at_ns))?; }
        match self.payload {            wiretap::mod_Envelope::OneOfpayload::snapshot_start(ref m) => { w.write_with_tag(90, |w| w.write_message(m))? },
            wiretap::mod_Envelope::OneOfpayload::snapshot_end(ref m) => { w.write_with_tag(98, |w| w.write_message(m))? },
            wiretap::mod_Envelope::OneOfpayload::string_def(ref m) => { w.write_with_tag(162, |w| w.write_message(m))? },
            wiretap::mod_Envelope::OneOfpayload::peer_upsert(ref m) => { w.write_with_tag(170, |w| w.write_message(m))? },
            wiretap::mod_Envelope::OneOfpayload::connection_upsert(ref m) => { w.write_with_tag(178, |w| w.write_message(m))? },
            wiretap::mod_Envelope::OneOfpayload::connection_closed(ref m) => { w.write_with_tag(186, |w| w.write_message(m))? },
            wiretap::mod_Envelope::OneOfpayload::stream_upsert(ref m) => { w.write_with_tag(194, |w| w.write_message(m))? },
            wiretap::mod_Envelope::OneOfpayload::stream_closed(ref m) => { w.write_with_tag(202, |w| w.write_message(m))? },
            wiretap::mod_Envelope::OneOfpayload::stream_chunk(ref m) => { w.write_with_tag(210, |w| w.write_message(m))? },
            wiretap::mod_Envelope::OneOfpayload::None => {},
    }        Ok(())
    }
}

pub mod mod_Envelope {

use super::*;

#[derive(Debug, PartialEq, Clone)]
pub enum OneOfpayload {
    snapshot_start(wiretap::SnapshotStart),
    snapshot_end(wiretap::SnapshotEnd),
    string_def(wiretap::StringDef),
    peer_upsert(wiretap::PeerUpsert),
    connection_upsert(wiretap::ConnectionUpsert),
    connection_closed(wiretap::ConnectionClosed),
    stream_upsert(wiretap::StreamUpsert),
    stream_closed(wiretap::StreamClosed),
    stream_chunk(wiretap::StreamChunk),
    None,
}

impl Default for OneOfpayload {
    fn default() -> Self {
        OneOfpayload::None
    }
}

}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ClientHello {
    pub protocol_version: u32,
    pub peer_id: Vec<u8>,
    pub client_name: String,
    pub boot_id: Vec<u8>,
    pub started_at_ns: i64,
}

impl<'a> MessageRead<'a> for ClientHello {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.protocol_version = r.read_uint32(bytes)?,
                Ok(18) => msg.peer_id = r.read_bytes(bytes)?.to_owned(),
                Ok(26) => msg.client_name = r.read_string(bytes)?.to_owned(),
                Ok(34) => msg.boot_id = r.read_bytes(bytes)?.to_owned(),
                Ok(40) => msg.started_at_ns = r.read_int64(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for ClientHello {
    fn get_size(&self) -> usize {
        0
        + if self.protocol_version == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.protocol_version) as u64) }
        + if self.peer_id.is_empty() { 0 } else { 1 + sizeof_len((&self.peer_id).len()) }
        + if self.client_name == String::default() { 0 } else { 1 + sizeof_len((&self.client_name).len()) }
        + if self.boot_id.is_empty() { 0 } else { 1 + sizeof_len((&self.boot_id).len()) }
        + if self.started_at_ns == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.started_at_ns) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.protocol_version != 0u32 { w.write_with_tag(8, |w| w.write_uint32(*&self.protocol_version))?; }
        if !self.peer_id.is_empty() { w.write_with_tag(18, |w| w.write_bytes(&**&self.peer_id))?; }
        if self.client_name != String::default() { w.write_with_tag(26, |w| w.write_string(&**&self.client_name))?; }
        if !self.boot_id.is_empty() { w.write_with_tag(34, |w| w.write_bytes(&**&self.boot_id))?; }
        if self.started_at_ns != 0i64 { w.write_with_tag(40, |w| w.write_int64(*&self.started_at_ns))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ServerHello {
    pub protocol_version: u32,
    pub source_id: String,
    pub last_acked_seq: u64,
}

impl<'a> MessageRead<'a> for ServerHello {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.protocol_version = r.read_uint32(bytes)?,
                Ok(18) => msg.source_id = r.read_string(bytes)?.to_owned(),
                Ok(24) => msg.last_acked_seq = r.read_uint64(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for ServerHello {
    fn get_size(&self) -> usize {
        0
        + if self.protocol_version == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.protocol_version) as u64) }
        + if self.source_id == String::default() { 0 } else { 1 + sizeof_len((&self.source_id).len()) }
        + if self.last_acked_seq == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.last_acked_seq) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.protocol_version != 0u32 { w.write_with_tag(8, |w| w.write_uint32(*&self.protocol_version))?; }
        if self.source_id != String::default() { w.write_with_tag(18, |w| w.write_string(&**&self.source_id))?; }
        if self.last_acked_seq != 0u64 { w.write_with_tag(24, |w| w.write_uint64(*&self.last_acked_seq))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct SnapshotStart { }

impl<'a> MessageRead<'a> for SnapshotStart {
    fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for SnapshotStart { }

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct SnapshotEnd { }

impl<'a> MessageRead<'a> for SnapshotEnd {
    fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for SnapshotEnd { }

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct StringDef {
    pub id: u32,
    pub value: String,
}

impl<'a> MessageRead<'a> for StringDef {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.id = r.read_uint32(bytes)?,
                Ok(18) => msg.value = r.read_string(bytes)?.to_owned(),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for StringDef {
    fn get_size(&self) -> usize {
        0
        + if self.id == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.id) as u64) }
        + if self.value == String::default() { 0 } else { 1 + sizeof_len((&self.value).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.id != 0u32 { w.write_with_tag(8, |w| w.write_uint32(*&self.id))?; }
        if self.value != String::default() { w.write_with_tag(18, |w| w.write_string(&**&self.value))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct PeerUpsert {
    pub peer_alias: u64,
    pub peer_id: Vec<u8>,
}

impl<'a> MessageRead<'a> for PeerUpsert {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.peer_alias = r.read_uint64(bytes)?,
                Ok(18) => msg.peer_id = r.read_bytes(bytes)?.to_owned(),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for PeerUpsert {
    fn get_size(&self) -> usize {
        0
        + if self.peer_alias == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.peer_alias) as u64) }
        + if self.peer_id.is_empty() { 0 } else { 1 + sizeof_len((&self.peer_id).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.peer_alias != 0u64 { w.write_with_tag(8, |w| w.write_uint64(*&self.peer_alias))?; }
        if !self.peer_id.is_empty() { w.write_with_tag(18, |w| w.write_bytes(&**&self.peer_id))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ConnectionUpsert {
    pub conn_alias: u64,
    pub peer_alias: u64,
    pub remote_addr: String,
    pub local_addr: String,
    pub direction: wiretap::Direction,
    pub transport_id: u32,
    pub security_id: u32,
    pub muxer_id: u32,
    pub opened_at_ns: i64,
}

impl<'a> MessageRead<'a> for ConnectionUpsert {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.conn_alias = r.read_uint64(bytes)?,
                Ok(16) => msg.peer_alias = r.read_uint64(bytes)?,
                Ok(26) => msg.remote_addr = r.read_string(bytes)?.to_owned(),
                Ok(34) => msg.local_addr = r.read_string(bytes)?.to_owned(),
                Ok(40) => msg.direction = r.read_enum(bytes)?,
                Ok(48) => msg.transport_id = r.read_uint32(bytes)?,
                Ok(56) => msg.security_id = r.read_uint32(bytes)?,
                Ok(64) => msg.muxer_id = r.read_uint32(bytes)?,
                Ok(72) => msg.opened_at_ns = r.read_int64(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for ConnectionUpsert {
    fn get_size(&self) -> usize {
        0
        + if self.conn_alias == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.conn_alias) as u64) }
        + if self.peer_alias == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.peer_alias) as u64) }
        + if self.remote_addr == String::default() { 0 } else { 1 + sizeof_len((&self.remote_addr).len()) }
        + if self.local_addr == String::default() { 0 } else { 1 + sizeof_len((&self.local_addr).len()) }
        + if self.direction == wiretap::Direction::DIRECTION_UNKNOWN { 0 } else { 1 + sizeof_varint(*(&self.direction) as u64) }
        + if self.transport_id == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.transport_id) as u64) }
        + if self.security_id == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.security_id) as u64) }
        + if self.muxer_id == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.muxer_id) as u64) }
        + if self.opened_at_ns == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.opened_at_ns) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.conn_alias != 0u64 { w.write_with_tag(8, |w| w.write_uint64(*&self.conn_alias))?; }
        if self.peer_alias != 0u64 { w.write_with_tag(16, |w| w.write_uint64(*&self.peer_alias))?; }
        if self.remote_addr != String::default() { w.write_with_tag(26, |w| w.write_string(&**&self.remote_addr))?; }
        if self.local_addr != String::default() { w.write_with_tag(34, |w| w.write_string(&**&self.local_addr))?; }
        if self.direction != wiretap::Direction::DIRECTION_UNKNOWN { w.write_with_tag(40, |w| w.write_enum(*&self.direction as i32))?; }
        if self.transport_id != 0u32 { w.write_with_tag(48, |w| w.write_uint32(*&self.transport_id))?; }
        if self.security_id != 0u32 { w.write_with_tag(56, |w| w.write_uint32(*&self.security_id))?; }
        if self.muxer_id != 0u32 { w.write_with_tag(64, |w| w.write_uint32(*&self.muxer_id))?; }
        if self.opened_at_ns != 0i64 { w.write_with_tag(72, |w| w.write_int64(*&self.opened_at_ns))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ConnectionClosed {
    pub conn_alias: u64,
    pub closed_at_ns: i64,
}

impl<'a> MessageRead<'a> for ConnectionClosed {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.conn_alias = r.read_uint64(bytes)?,
                Ok(16) => msg.closed_at_ns = r.read_int64(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for ConnectionClosed {
    fn get_size(&self) -> usize {
        0
        + if self.conn_alias == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.conn_alias) as u64) }
        + if self.closed_at_ns == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.closed_at_ns) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.conn_alias != 0u64 { w.write_with_tag(8, |w| w.write_uint64(*&self.conn_alias))?; }
        if self.closed_at_ns != 0i64 { w.write_with_tag(16, |w| w.write_int64(*&self.closed_at_ns))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct StreamUpsert {
    pub stream_alias: u64,
    pub conn_alias: u64,
    pub direction: wiretap::Direction,
    pub protocol_id: u32,
    pub opened_at_ns: i64,
}

impl<'a> MessageRead<'a> for StreamUpsert {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.stream_alias = r.read_uint64(bytes)?,
                Ok(16) => msg.conn_alias = r.read_uint64(bytes)?,
                Ok(24) => msg.direction = r.read_enum(bytes)?,
                Ok(32) => msg.protocol_id = r.read_uint32(bytes)?,
                Ok(40) => msg.opened_at_ns = r.read_int64(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for StreamUpsert {
    fn get_size(&self) -> usize {
        0
        + if self.stream_alias == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.stream_alias) as u64) }
        + if self.conn_alias == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.conn_alias) as u64) }
        + if self.direction == wiretap::Direction::DIRECTION_UNKNOWN { 0 } else { 1 + sizeof_varint(*(&self.direction) as u64) }
        + if self.protocol_id == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.protocol_id) as u64) }
        + if self.opened_at_ns == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.opened_at_ns) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.stream_alias != 0u64 { w.write_with_tag(8, |w| w.write_uint64(*&self.stream_alias))?; }
        if self.conn_alias != 0u64 { w.write_with_tag(16, |w| w.write_uint64(*&self.conn_alias))?; }
        if self.direction != wiretap::Direction::DIRECTION_UNKNOWN { w.write_with_tag(24, |w| w.write_enum(*&self.direction as i32))?; }
        if self.protocol_id != 0u32 { w.write_with_tag(32, |w| w.write_uint32(*&self.protocol_id))?; }
        if self.opened_at_ns != 0i64 { w.write_with_tag(40, |w| w.write_int64(*&self.opened_at_ns))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct StreamClosed {
    pub stream_alias: u64,
    pub closed_at_ns: i64,
    pub reason: wiretap::CloseReason,
}

impl<'a> MessageRead<'a> for StreamClosed {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.stream_alias = r.read_uint64(bytes)?,
                Ok(16) => msg.closed_at_ns = r.read_int64(bytes)?,
                Ok(24) => msg.reason = r.read_enum(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for StreamClosed {
    fn get_size(&self) -> usize {
        0
        + if self.stream_alias == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.stream_alias) as u64) }
        + if self.closed_at_ns == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.closed_at_ns) as u64) }
        + if self.reason == wiretap::CloseReason::CLOSE_REASON_UNKNOWN { 0 } else { 1 + sizeof_varint(*(&self.reason) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.stream_alias != 0u64 { w.write_with_tag(8, |w| w.write_uint64(*&self.stream_alias))?; }
        if self.closed_at_ns != 0i64 { w.write_with_tag(16, |w| w.write_int64(*&self.closed_at_ns))?; }
        if self.reason != wiretap::CloseReason::CLOSE_REASON_UNKNOWN { w.write_with_tag(24, |w| w.write_enum(*&self.reason as i32))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct StreamChunk {
    pub stream_alias: u64,
    pub direction: wiretap::Direction,
    pub data: Vec<u8>,
}

impl<'a> MessageRead<'a> for StreamChunk {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.stream_alias = r.read_uint64(bytes)?,
                Ok(16) => msg.direction = r.read_enum(bytes)?,
                Ok(26) => msg.data = r.read_bytes(bytes)?.to_owned(),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for StreamChunk {
    fn get_size(&self) -> usize {
        0
        + if self.stream_alias == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.stream_alias) as u64) }
        + if self.direction == wiretap::Direction::DIRECTION_UNKNOWN { 0 } else { 1 + sizeof_varint(*(&self.direction) as u64) }
        + if self.data.is_empty() { 0 } else { 1 + sizeof_len((&self.data).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.stream_alias != 0u64 { w.write_with_tag(8, |w| w.write_uint64(*&self.stream_alias))?; }
        if self.direction != wiretap::Direction::DIRECTION_UNKNOWN { w.write_with_tag(16, |w| w.write_enum(*&self.direction as i32))?; }
        if !self.data.is_empty() { w.write_with_tag(26, |w| w.write_bytes(&**&self.data))?; }
        Ok(())
    }
}

