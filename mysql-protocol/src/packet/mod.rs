use crate::error::Error;

use bytes::{Buf, BufMut, Bytes};

pub use handshake::{Handshake, HandshakeResponse};
use mysql_common::constants::CapabilityFlags;

mod handshake;
mod utils;
mod generic;

pub trait Packet: Sized {
    fn read(buf: &mut impl Buf, capabilities: CapabilityFlags) -> Result<Self, Error>;
    fn write(&self, buf: &mut impl BufMut, capabilities: CapabilityFlags) -> Result<(), Error>;
    fn size_hint(&self) -> Option<usize> { None }
}

impl Packet for Bytes {
    fn read(buf: &mut impl Buf, _: CapabilityFlags) -> Result<Bytes, Error> {
        Ok(buf.copy_to_bytes(buf.remaining()))
    }

    fn write(&self, buf: &mut impl BufMut, _ : CapabilityFlags) -> Result<(), Error> {
        buf.put_slice(self.bytes());
        Ok(())
    }

    fn size_hint(&self) -> Option<usize> { Some(self.len()) }
}
