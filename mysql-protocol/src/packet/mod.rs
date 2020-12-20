use crate::error::Error;

mod handshake;

use bytes::Bytes;
pub use handshake::Handshake;

pub trait Packet: Sized {
    fn read<R: std::io::BufRead>(reader: &mut R) -> Result<Self, Error>;

    /// Attempts to read the packet out of the provided `Buf`.
    ///
    /// If `Ok` is returned, the buffer will have been advanced past the packet.
    /// If `Err` is returned, the buffer will **not** have been advanced at all.
    fn try_read<B: bytes::Buf>(buf: &mut B) -> Result<Self, Error> {
        if buf.remaining() < 4 {
            return Err(Error::DataIncomplete);
        }

        let header = &buf.bytes()[0..4];
        let payload_len =
            (header[0] as usize) | ((header[1] as usize) << 8) | ((header[2] as usize) << 16);

        if buf.remaining() < 4 + payload_len {
            return Err(Error::DataIncomplete);
        }

        match Self::read(&mut &buf.bytes()[4..(4 + payload_len)]) {
            Ok(p) => {
                buf.advance(4 + payload_len);
                Ok(p)
            }
            Err(e) => Err(e),
        }
    }

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
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Err(Error::InvalidPacket("packet ended prematurely reading {} bytes".into())),
        Err(e) => Err(e.into())
    }
}

fn read_string<R: std::io::BufRead>(buf: &mut R, allow_missing_terminator: bool) -> Result<String, Error> {
    let mut bytes = Vec::new();
    buf.read_until(0, &mut bytes)?;

    if let Some(0u8) = bytes.last() {
        bytes.pop();
    } else if !allow_missing_terminator {
        return Err(Error::InvalidPacket("string is missing null-terminator".into()))
    }

    match std::str::from_utf8(&bytes) {
        Ok(s) => Ok(s.into()),
        Err(_) => Err(Error::InvalidPacket("string is not valid utf-8".into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use bytes::Bytes;

    use super::Packet;

    #[test]
    pub fn try_parse_returns_incomplete_if_insufficient_space_in_provided_buffer() {
        let mut data = Bytes::from_static(&[]);
        assert_eq!(Error::DataIncomplete, Bytes::try_read(&mut data).unwrap_err());
        let mut data = Bytes::from_static(&[0, 0]);
        assert_eq!(Error::DataIncomplete, Bytes::try_read(&mut data).unwrap_err());
        let mut data = Bytes::from_static(&[4, 0, 0, 0, 0, 0]);
        assert_eq!(Error::DataIncomplete, Bytes::try_read(&mut data).unwrap_err());
    }

    #[test]
    pub fn try_parse_returns_result_of_parsing_if_sufficient_space_in_buffer() {
        let mut data = Bytes::from_static(&[4, 0, 0, 0, 1, 2, 3, 4]);
        assert_eq!(vec![1u8, 2u8, 3u8, 4u8], Bytes::try_read(&mut data).unwrap());

        let mut data = Bytes::from_static(&[4, 0, 0, 0, 1, 2, 3, 4]);
        assert!(FailToParse::try_read(&mut data).is_err())
    }

    struct FailToParse;

    impl Packet for FailToParse {
        fn read<R: std::io::Read>(_: &mut R) -> Result<FailToParse, Error> {
            Err(Error::Other("it's bad".into()))
        }

        fn write<W: std::io::Write>(&self, w: &mut W) -> Result<(), Error> {
            Err(Error::Other("it's bad".into()))
        }
    }
}
