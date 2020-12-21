use mysql_common::constants::{CapabilityFlags, StatusFlags};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{error::{Error, ErrorKind}, packet::Handshake};

mod transport;

use transport::Transport;

#[derive(Debug)]
struct ConnectionState {
    capability_flags: CapabilityFlags,
    status_flags: StatusFlags,
    connection_id: usize,
    character_set: u8,
    server_version: String,
}

#[derive(Debug)]
pub struct Connection<S: AsyncRead + AsyncWrite + Unpin> {
    transport: Transport<S>,
    state: ConnectionState
}

impl<S: AsyncRead + AsyncWrite + Unpin> Connection<S> {
    pub fn new(stream: S) -> Connection<S> {
        Connection {
           transport: Transport::new(stream),
           state: ConnectionState {
               capability_flags: CapabilityFlags::from_bits_truncate(0),
               status_flags: StatusFlags::from_bits_truncate(0),
               connection_id: 0,
               character_set: 0,
               server_version: "".into(),
           }
        }
    }

    pub async fn connect(&mut self) -> Result<(), Error> {
        let handshake = self.transport
            .read_packet::<Handshake>()
            .await?
            .ok_or(Error::new(
                ErrorKind::ProtocolError,
                "expected a handshake packet"))?;

        if handshake.protocol_version != 10 {
            return Err(Error::new(
                ErrorKind::UnsupportedProtocol {requested: handshake.protocol_version, required: 10},
                format!("server requested protocol version {} but this client only supports 10", handshake.protocol_version)))
        }

        if !handshake.capability_flags.contains(CapabilityFlags::CLIENT_PROTOCOL_41) {
            return Err(Error::new(
                ErrorKind::ProtocolError,
                "server must use protocol 4.1"))
        }

        // Read server values out
        self.state.capability_flags = handshake.capability_flags;
        self.state.status_flags = handshake.status_flags;
        self.state.connection_id = handshake.connection_id as usize;
        self.state.status_flags = handshake.status_flags;
        self.state.server_version = handshake.server_version;

        Ok(())
    }
}
