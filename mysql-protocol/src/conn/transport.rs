use std::{fmt::Debug, todo};

use crate::{error::{Error, ErrorKind}, packet::Packet};
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

const BUFFER_SIZE: usize = 4096;

pub struct Transport<S: AsyncRead + AsyncWrite + Unpin> {
    stream: S,
    buffer: BytesMut,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Debug for Transport<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<<Transport>>")
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> Transport<S> {
    pub fn new(stream: S) -> Transport<S> {
        Transport {
            stream,
            buffer: BytesMut::with_capacity(BUFFER_SIZE),
        }
    }

    pub async fn read_packet<P: Packet>(&mut self) -> Result<Option<P>, Error> {
        loop {
            // Attempt to parse from the buffer
            if let Some(packet) = self.parse_packet::<P>()? {
                return Ok(Some(packet));
            }

            if self.stream.read_buf(&mut self.buffer).await? == 0 {
                // No more data, the remote closed the connection.
                if self.buffer.is_empty() {
                    // Clean shutdown, there's no incomplete packets in the buffer
                    return Ok(None);
                } else {
                    // We were shut down with incomplete packets still in the buffer
                    return Err(Error::new(ErrorKind::ConnectionReset, "connection reset by peer"));
                }
            }
        }
    }

    pub async fn write_packet<P: Packet>(&mut self, _packet: P) -> Result<(), Error> {
        todo!()
    }

    fn parse_packet<P: Packet>(&mut self) -> Result<Option<P>, Error> {
        let mut buf = std::io::Cursor::new(&self.buffer[..]);

        match P::try_read(&mut buf) {
            Ok(packet) => {
                let len = buf.position() as usize;
                self.buffer.advance(len);
                Ok(Some(packet))
            }
            Err(e) if e.kind() == ErrorKind::DataIncomplete => {
                // Need more data
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use bytes::Bytes;

    use crate::error::ErrorKind;
    use super::Transport;

    #[tokio::test]
    pub async fn read_packet_reads_next_packet() {
        let data = vec![4u8, 0, 0, 0, 1, 2, 3, 4, 4u8, 0, 0, 0, 5, 6, 7, 8];
        let mut conn = Transport::new(Cursor::new(data));
        let packet = conn.read_packet::<Bytes>().await.unwrap().unwrap();
        assert_eq!(&[1, 2, 3, 4], packet.as_ref());
        let packet = conn.read_packet::<Bytes>().await.unwrap().unwrap();
        assert_eq!(&[5, 6, 7, 8], packet.as_ref());
    }

    #[tokio::test]
    pub async fn read_packet_returns_none_if_no_more_data_and_no_incomplete_packet() {
        let data = vec![4u8, 0, 0, 0, 1, 2, 3, 4];
        let mut conn = Transport::new(Cursor::new(data));
        assert!(conn.read_packet::<Bytes>().await.unwrap().is_some());
        assert!(conn.read_packet::<Bytes>().await.unwrap().is_none());
    }

    #[tokio::test]
    pub async fn read_packet_returns_connection_reset_if_no_more_data_and_incomplete_packet() {
        let data = vec![4u8, 0];
        let mut conn = Transport::new(Cursor::new(data));
        assert_eq!(ErrorKind::ConnectionReset, conn.read_packet::<Bytes>().await.unwrap_err().kind())
    }
}
