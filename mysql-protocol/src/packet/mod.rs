use crate::error::{Error, ErrorKind};

use bytes::Bytes;
use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};

pub use handshake::{Handshake, HandshakeResponse};

mod handshake;

pub trait Packet: Sized {
    fn read<R: std::io::BufRead>(reader: &mut R) -> Result<Self, Error>;
    fn write<W: std::io::Write>(&self, w: &mut W) -> Result<(), Error>;
    fn size_hint(&self) -> Option<usize> { None }
}

impl Packet for Bytes {
    fn read<R: std::io::BufRead>(reader: &mut R) -> Result<Bytes, Error> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(data.into())
    }

    fn write<W: std::io::Write>(&self, w: &mut W) -> Result<(), Error> {
        w.write_all(&self).map_err(|e| e.into())
    }

    fn size_hint(&self) -> Option<usize> { Some(self.len()) }
}

fn read_bytes<R: std::io::Read>(buf: &mut R, count: usize) -> Result<Vec<u8>, Error> {
    let mut bytes = vec![0u8; count];
    match buf.read_exact(&mut bytes) {
        Ok(()) => Ok(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Err(Error::new(
            ErrorKind::InvalidPacket,
            "packet ended prematurely reading {} bytes")),
        Err(e) => Err(e.into())
    }
}

fn read_string<R: std::io::BufRead>(buf: &mut R, allow_missing_terminator: bool) -> Result<String, Error> {
    let mut bytes = Vec::new();
    buf.read_until(0, &mut bytes)?;

    if let Some(0u8) = bytes.last() {
        bytes.pop();
    } else if !allow_missing_terminator {
        return Err(Error::new(
            ErrorKind::InvalidPacket,
            "string is missing null-terminator"))
    }

    match std::str::from_utf8(&bytes) {
        Ok(s) => Ok(s.into()),
        Err(_) => Err(Error::new(
            ErrorKind::InvalidPacket,
            "string is not valid utf-8"))
    }
}

fn write_string<W: std::io::Write>(buf: &mut W, s: &str) -> Result<(), Error> {
    buf.write_all(s.as_bytes())?;
    buf.write_u8(0)?;
    Ok(())
}

fn read_lenenc_int<R: std::io::Read>(r: &mut R) -> Result<u64, Error> {
    match r.read_u8()? {
        0xFE => Ok(r.read_u64::<LE>()?),
        0xFD => Ok(r.read_uint::<LE>(3)?),
        0xFC => Ok(r.read_u16::<LE>()? as u64),
        x if x < 0xFB => Ok(x as u64),
        x => Err(Error::new(
            ErrorKind::InvalidPacket,
            format!("Invalid lenenc integer: 0x{:02X}", x))),
    }
}

fn write_lenenc_int<W: std::io::Write>(buf: &mut W, val: u64) -> Result<usize, Error> {
    if val < 128 {
        buf.write_u8(val as u8)?;
        Ok(1)
    } else if val < (2 as u64).pow(16) {
        buf.write_u8(0xFC)?;
        buf.write_u16::<LE>(val as u16)?;
        Ok(3)
    } else if val < (2 as u64).pow(24) {
        buf.write_u8(0xFD)?;
        buf.write_uint::<LE>(val as u64, 3)?;
        Ok(4)
    } else {
        buf.write_u8(0xFE)?;
        buf.write_u64::<LE>(val as u64)?;
        Ok(9)
    }
}

#[cfg(test)]
mod tests {
    use crate::error::{Error, ErrorKind};
    use bytes::Bytes;

    use super::{Packet, read_lenenc_int, write_lenenc_int};

    #[test]
    pub fn can_read_and_write_lenenc_ints() {
        fn rw_test(val: u64, mut bytes: &'static [u8]) {
            let mut write: Vec<u8> = Vec::new();
            write_lenenc_int(&mut write, val).unwrap();
            assert_eq!(bytes, write.as_slice());

            let read = read_lenenc_int(&mut bytes).unwrap();
            assert_eq!(read, val);
        }

        rw_test(0x7F, &[0x7F]);
        rw_test(0xBEEF,&[0xFC, 0xEF, 0xBE]);
        rw_test(0xBEEFCA,&[0xFD, 0xCA, 0xEF, 0xBE]);
        rw_test(0xBEEFCAFE,&[0xFE, 0xFE, 0xCA, 0xEF, 0xBE, 0, 0, 0, 0]);
        rw_test(0xBEEFCAFEBEEFCAFE,&[0xFE, 0xFE, 0xCA, 0xEF, 0xBE, 0xFE, 0xCA, 0xEF, 0xBE]);
    }
}
