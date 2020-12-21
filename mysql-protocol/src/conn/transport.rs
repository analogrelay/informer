use std::{fmt::Debug, todo};

use crate::{error::{Error, ErrorKind}, packet::Packet};
use byteorder::{ByteOrder, LittleEndian};
use bytes::{Buf, Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufWriter};

const BUFFER_SIZE: usize = 4096;

pub struct Transport<S: AsyncRead + AsyncWrite + Unpin> {
    stream: S,
    buffer: BytesMut,
    packet: Option<Bytes>,
    sequence_id: u8
}

impl<S: AsyncRead + AsyncWrite + Unpin> Debug for Transport<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<<Transport>>")
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> Transport<S> {
    pub fn new(stream: S) -> Transport<S> {
        Transport {
            stream: stream,
            buffer: BytesMut::with_capacity(BUFFER_SIZE),
            packet: None,
            sequence_id: 0
        }
    }

    /// Waits for the next packet and returns a reference to the buffer and sequence ID.
    ///
    /// This does not advance the buffer to read more data, so repeated calls to this
    /// method (without calling next_packet or read_packet) will return the same result.
    pub async fn peek_packet(&mut self) -> Result<Option<impl Buf>, Error> {
        self.fill_packet_buffer().await?;
        self.get_next_packet()
    }

    /// Advance the buffer to skip the current packet without reading it.
    pub fn next_packet(&mut self) {
        self.packet = None;
    }

    /// Waits for the next packet, reads it into the provided structure, and advances the buffer
    pub async fn read_packet<P: Packet>(&mut self) -> Result<Option<P>, Error> {
        if let Some(packet) = self.fill_packet_buffer().await? {
            self.next_packet();
            Ok(Some(P::read(&mut packet.reader())?))
        } else {
            Ok(None)
        }
    }

    pub async fn write_packet<P: Packet>(&mut self, packet: P) -> Result<(), Error> {
        let mut output = if let Some(hint) = packet.size_hint() {
            Vec::with_capacity(hint)
        } else {
            Vec::new()
        };
        packet.write(&mut output)?;

        let mut header = [0u8; 4];
        LittleEndian::write_uint(&mut header[0..3], output.len() as u64, 3);

        header[3] = self.next_sequence_id();

        self.stream.write_all(&header).await?;
        self.stream.write_all(&output).await?;
        Ok(())
    }

    fn next_sequence_id(&mut self) -> u8 {
        let next = self.sequence_id;
        self.sequence_id = self.sequence_id.wrapping_add(1);
        next
    }

    fn check_sequence_id(&mut self, recieved_sequence_id: u8) -> Result<(), Error> {
        if self.sequence_id != recieved_sequence_id {
            Err(Error::new(
                ErrorKind::ProtocolError,
                format!("messages out of sequence, expecting #{} but received #{}", self.sequence_id, recieved_sequence_id)))
        } else {
            self.sequence_id = self.sequence_id.wrapping_add(1);
            Ok(())
        }
    }

    async fn fill_packet_buffer(&mut self) -> Result<Option<Bytes>, Error> {
        loop {
            if let Some(p) = self.get_next_packet()? {
                return Ok(Some(p))
            }

            if self.stream.read_buf(&mut self.buffer).await? == 0 {
                // No more data, the remote closed the connection.
                if self.buffer.is_empty() {
                    // Clean shutdown, there's no incomplete packets in the buffer
                    return Ok(None)
                } else {
                    // We were shut down with incomplete packets still in the buffer
                    return Err(Error::new(ErrorKind::ConnectionReset, "connection reset by peer"))
                }
            }
        }
    }

    fn get_next_packet(&mut self) -> Result<Option<Bytes>, Error> {
        if self.packet.is_none() {
            if self.buffer.remaining() < 4 {
                return Ok(None);
            }
            let header = &self.buffer.bytes()[0..4];
            let payload_len = LittleEndian::read_uint(&header[0..3], 3) as usize;
            self.check_sequence_id(header[3])?;

            self.packet = if self.buffer.remaining() < 4 + payload_len {
                None
            }
            else {
                let mut packet_buffer = self.buffer.split_to(4 + payload_len);
                let packet_buffer = packet_buffer.split_off(4).freeze();
                Some(packet_buffer)
            };
        }
        Ok(self.packet.as_ref().map(|p| p.clone()))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use bytes::{Buf, Bytes};

    use crate::error::ErrorKind;
    use super::Transport;

    #[tokio::test]
    pub async fn read_packet_reads_next_packet() {
        let data = vec![4u8, 0, 0, 0, 1, 2, 3, 4, 4u8, 0, 0, 1, 5, 6, 7, 8];
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

    #[tokio::test]
    pub async fn get_next_packet_returns_payload_and_sequence_id_if_packet_in_buffer() {
        let data = vec![4u8, 0, 0, 0, 1, 2, 3, 4];
        let mut conn = Transport::new(Cursor::new(data));
        assert!(conn.fill_packet_buffer().await.unwrap().is_some());
        assert_eq!(Ok(Some(Bytes::from_static(&[1u8, 2, 3, 4]))), conn.get_next_packet());
    }

    #[tokio::test]
    pub async fn peek_packet_returns_next_packet() {
        let data = vec![4u8, 0, 0, 0, 1, 2, 3, 4];
        let mut conn = Transport::new(Cursor::new(data));
        let packet = conn.peek_packet().await.unwrap().unwrap();
        assert_eq!(&[1, 2, 3, 4], packet.bytes());
    }
}
