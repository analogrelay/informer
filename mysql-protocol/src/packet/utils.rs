use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};

use crate::error::{Error, ErrorKind};

pub trait ReadMySqlExt: std::io::Read {
    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![0u8; count];
        match self.read_exact(&mut bytes) {
            Ok(()) => Ok(bytes),
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Err(Error::new(
                ErrorKind::InvalidPacket,
                "packet ended prematurely reading {} bytes")),
            Err(e) => Err(e.into())
        }
    }

    fn read_lenenc_bytes(&mut self) -> Result<Vec<u8>, Error> {
        let len = self.read_lenenc_int()?;
        self.read_bytes(len as usize)
    }

    fn read_lenenc_string(&mut self) -> Result<String, Error> {
        let bytes = self.read_lenenc_bytes()?;
        match String::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(_) => Err(Error::new(
                ErrorKind::ProtocolError,
                "string is not valid utf-8"))
        }
    }

    fn read_lenenc_int(&mut self) -> Result<u64, Error> {
        match self.read_u8()? {
            0xFE => Ok(self.read_u64::<LE>()?),
            0xFD => Ok(self.read_uint::<LE>(3)?),
            0xFC => Ok(self.read_u16::<LE>()? as u64),
            x if x < 0xFB => Ok(x as u64),
            x => Err(Error::new(
                ErrorKind::InvalidPacket,
                format!("invalid lenenc integer: 0x{:02X}", x))),
        }
    }
}

impl<R: std::io::Read + ?Sized> ReadMySqlExt for R {}

pub trait BufReadMySqlExt: std::io::BufRead {
    fn read_string(&mut self, allow_missing_terminator: bool) -> Result<String, Error> {
        let mut bytes = Vec::new();
        self.read_until(0, &mut bytes)?;

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
}

impl<R: std::io::BufRead + ?Sized> BufReadMySqlExt for R {}

pub trait WriteMySqlExt: std::io::Write {
    fn write_string(&mut self, s: &str) -> Result<(), Error> {
        self.write_all(s.as_bytes())?;
        self.write_u8(0)?;
        Ok(())
    }

    fn write_lenenc_bytes(&mut self, bytes: &[u8]) -> Result<usize, Error> {
        let lenlen = self.write_lenenc_int(bytes.len() as u64)?;
        self.write_all(bytes)?;
        Ok(lenlen + bytes.len())
    }

    fn write_lenenc_string(&mut self, str: &str) -> Result<usize, Error> {
        let bytes = str.as_bytes();
        self.write_lenenc_bytes(bytes)
    }

    fn write_lenenc_int(&mut self, val: u64) -> Result<usize, Error> {
        if val < 128 {
            self.write_u8(val as u8)?;
            Ok(1)
        } else if val < (2 as u64).pow(16) {
            self.write_u8(0xFC)?;
            self.write_u16::<LE>(val as u16)?;
            Ok(3)
        } else if val < (2 as u64).pow(24) {
            self.write_u8(0xFD)?;
            self.write_uint::<LE>(val as u64, 3)?;
            Ok(4)
        } else {
            self.write_u8(0xFE)?;
            self.write_u64::<LE>(val as u64)?;
            Ok(9)
        }
    }
}

impl<W: std::io::Write + ?Sized> WriteMySqlExt for W {}

#[cfg(test)]
mod tests {
    use super::{ReadMySqlExt, WriteMySqlExt};

    #[test]
    pub fn can_read_and_write_lenenc_ints() {
        fn rw_test(val: u64, mut bytes: &'static [u8]) {
            let mut write: Vec<u8> = Vec::new();
            write.write_lenenc_int(val).unwrap();
            assert_eq!(bytes, write.as_slice());

            let read = bytes.read_lenenc_int().unwrap();
            assert_eq!(read, val);
        }

        rw_test(0x7F, &[0x7F]);
        rw_test(0xBEEF,&[0xFC, 0xEF, 0xBE]);
        rw_test(0xBEEFCA,&[0xFD, 0xCA, 0xEF, 0xBE]);
        rw_test(0xBEEFCAFE,&[0xFE, 0xFE, 0xCA, 0xEF, 0xBE, 0, 0, 0, 0]);
        rw_test(0xBEEFCAFEBEEFCAFE,&[0xFE, 0xFE, 0xCA, 0xEF, 0xBE, 0xFE, 0xCA, 0xEF, 0xBE]);
    }
}
