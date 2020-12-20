use crate::error::Error;
use bytes::Bytes;

mod handshake;

pub use handshake::Handshake;

pub trait PacketReader {
    type Packet;

    fn parse(payload: &[u8]) -> Result<Self::Packet, Error>;

    /// Attempts to read the packet out of the provided `Buf`.
    ///
    /// If `Ok` is returned, the buffer will have been advanced past the packet.
    /// If `Err` is returned, the buffer will **not** have been advanced at all.
    fn try_read<B: bytes::Buf>(buf: &mut B) -> Result<Self::Packet, Error> {
        if buf.remaining() < 4 {
            return Err(Error::DataIncomplete);
        }

        let header = &buf.bytes()[0..4];
        let payload_len =
            (header[0] as usize) | ((header[1] as usize) << 8) | ((header[2] as usize) << 16);

        if buf.remaining() < 4 + payload_len {
            return Err(Error::DataIncomplete);
        }

        match Self::parse(&buf.bytes()[4..(4 + payload_len)]) {
            Ok(p) => {
                buf.advance(4 + payload_len);
                Ok(p)
            }
            Err(e) => Err(e),
        }
    }
}

pub struct Raw;

impl PacketReader for Raw {
    type Packet = Bytes;

    fn parse(payload: &[u8]) -> Result<Bytes, Error> {
        Ok(Bytes::copy_from_slice(payload))
    }
}

fn read_bytes<'a>(buf: &mut &'a [u8], count: usize) -> Result<&'a [u8], Error> {
    if buf.len() < count {
        Err(Error::InvalidPacket("packet is missing expected data".into()))
    } else {
        let ret = &buf[0..count];
        *buf = &buf[count..];
        Ok(ret)
    }
}

fn read_u8(buf: &mut &[u8]) -> Result<u8, Error> {
    let r = read_bytes(buf, 1)?;
    Ok(r[0])
}

fn read_u16(buf: &mut &[u8]) -> Result<u16, Error> {
    let r = read_bytes(buf, 2)?;
    Ok(
        (r[0] as u16) |
        ((r[1] as u16) << 8)
    )
}

fn read_u32(buf: &mut &[u8]) -> Result<u32, Error> {
    let r = read_bytes(buf, 4)?;
    Ok(
        (r[0] as u32) |
        ((r[1] as u32) << 8) |
        ((r[2] as u32) << 16) |
        ((r[3] as u32) << 24)
    )
}

fn read_string(buf: &mut &[u8], allow_missing_terminator: bool) -> Result<String, Error> {
    if buf.len() == 0 {
        return Err(Error::InvalidPacket("packet is missing expected data".into()))
    }
    for i in 0..buf.len() {
        if buf[i] == 0 {
            let b = &buf[0..i];
            *buf = &buf[i+1..];
            return Ok(strbytes_to_string(b)?)
        }
    }
    if allow_missing_terminator {
        strbytes_to_string(buf)
    } else {
        Err(Error::InvalidPacket("string is missing null-terminator".into()))
    }
}

fn strbytes_to_string(buf: &[u8]) -> Result<String, Error> {
    match std::str::from_utf8(buf) {
        Ok(s) => Ok(s.into()),
        Err(_) => Err(Error::InvalidPacket("string is not valid utf-8".into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::packet::{PacketReader, Raw};
    use bytes::Bytes;

    #[test]
    pub fn try_parse_returns_incomplete_if_insufficient_space_in_provided_buffer() {
        let mut data = Bytes::from_static(&[]);
        assert_eq!(Error::DataIncomplete, Raw::try_read(&mut data).unwrap_err());
        let mut data = Bytes::from_static(&[0, 0]);
        assert_eq!(Error::DataIncomplete, Raw::try_read(&mut data).unwrap_err());
        let mut data = Bytes::from_static(&[4, 0, 0, 0, 0, 0]);
        assert_eq!(Error::DataIncomplete, Raw::try_read(&mut data).unwrap_err());
    }

    #[test]
    pub fn try_parse_returns_result_of_parsing_if_sufficient_space_in_buffer() {
        let mut data = Bytes::from_static(&[4, 0, 0, 0, 1, 2, 3, 4]);
        assert_eq!(vec![1u8, 2u8, 3u8, 4u8], Raw::try_read(&mut data).unwrap());

        let mut data = Bytes::from_static(&[4, 0, 0, 0, 1, 2, 3, 4]);
        assert!(FailToParse::try_read(&mut data).is_err())
    }

    struct FailToParse;

    impl PacketReader for FailToParse {
        type Packet = ();

        fn parse(_: &[u8]) -> Result<Self::Packet, Error> {
            Err(Error::Other("it's bad".into()))
        }
    }
}
