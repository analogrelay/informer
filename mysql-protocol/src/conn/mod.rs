use auth::AuthPlugin;
use bytes::Bytes;
use mysql_common::{constants::{CapabilityFlags, StatusFlags, UTF8MB4_GENERAL_CI}};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{ConnectionOptions, error::{Error, ErrorKind}, packet::{Handshake, HandshakeResponse}};

mod auth;
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
    opts: ConnectionOptions,
    state: ConnectionState
}

impl<S: AsyncRead + AsyncWrite + Unpin> Connection<S> {
    pub fn new(stream: S, opts: Option<ConnectionOptions>) -> Connection<S> {
        Connection {
           transport: Transport::new(stream),
           opts: opts.unwrap_or_default(),
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
        self.state.capability_flags = handshake.capability_flags & self.get_client_caps();
        self.state.status_flags = handshake.status_flags;
        self.state.connection_id = handshake.connection_id as usize;
        self.state.status_flags = handshake.status_flags;
        self.state.server_version = handshake.server_version;

        // Generate auth data
        let auth_plugin = AuthPlugin::from_name(handshake.auth_plugin_name);
        if let AuthPlugin::Unknown(ref s) = auth_plugin {
            return Err(Error::new(
                ErrorKind::ClientIncapable,
                format!("the client is incapable of using auth plugin '{}'", s)
            ));
        }
        let auth_data = auth_plugin.generate_response(self.opts.password().map(|s| s.into()), handshake.auth_plugin_data.as_slice())?;

        // Write the response
        let response = HandshakeResponse {
            capability_flags: self.state.capability_flags,
            max_packet_size: 0x1000000,
            character_set: UTF8MB4_GENERAL_CI as u8,
            username: self.opts.username().unwrap_or("root").into(),
            auth_response: auth_data,
            initial_database: self.opts.initial_database().map(|s| s.into()),
            auth_plugin_name: auth_plugin.name().into(),
            attributes: self.get_connect_attrs()
        };
        self.transport.write_packet(response).await?;

        // Read a possible response (just the bytes for now)
        let response = self.transport.read_packet::<Bytes>().await?;
        println!("handshake response response: {:?}", response);

        Ok(())
    }

    fn get_connect_attrs(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn get_client_caps(&self) -> CapabilityFlags {
        let mut base_flags: CapabilityFlags = CapabilityFlags::CLIENT_PROTOCOL_41
            | CapabilityFlags::CLIENT_SECURE_CONNECTION
            | CapabilityFlags::CLIENT_LONG_PASSWORD
            | CapabilityFlags::CLIENT_TRANSACTIONS
            | CapabilityFlags::CLIENT_LOCAL_FILES
            | CapabilityFlags::CLIENT_MULTI_STATEMENTS
            | CapabilityFlags::CLIENT_MULTI_RESULTS
            | CapabilityFlags::CLIENT_PS_MULTI_RESULTS
            | CapabilityFlags::CLIENT_PLUGIN_AUTH
            | CapabilityFlags::CLIENT_CONNECT_ATTRS;
        base_flags | self.opts.get_capabilities()
    }
}
