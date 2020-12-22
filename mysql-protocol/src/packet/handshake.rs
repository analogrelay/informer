use std::{cmp::max, io::Write};

use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};
use bytes::{Buf, BufMut};
use mysql_common::constants::{CapabilityFlags, StatusFlags};

use crate::error::{Error, ErrorKind};

use super::{Packet, utils::*};

#[derive(Debug)]
pub struct Handshake {
    pub protocol_version: u8,
    pub server_version: String,
    pub connection_id: u32,
    pub auth_plugin_data: Vec<u8>,
    pub capability_flags: CapabilityFlags,
    pub character_set: u8,
    pub status_flags: StatusFlags,
    pub auth_plugin_name: String
}

impl Packet for Handshake {
    fn read(buf: &mut impl Buf, _: CapabilityFlags) -> Result<Handshake, Error> {
        let mut reader = buf.reader();

        let protocol_version = reader.read_u8()?;
        let server_version = reader.read_string(false)?;
        let connection_id = reader.read_u32::<LE>()?;
        let mut auth_plugin_data = reader.read_bytes(8)?;
        // skip the filler
        reader.read_u8()?;
        let mut capability_flags = CapabilityFlags::from_bits_truncate(reader.read_u16::<LE>()? as u32);
        let character_set = reader.read_u8()?;
        let status_flags = StatusFlags::from_bits_truncate(reader.read_u16::<LE>()?);
        capability_flags |= CapabilityFlags::from_bits_truncate((reader.read_u16::<LE>()? as u32) << 16);
        let auth_data_len = reader.read_u8()?;

        reader.read_bytes(10)?;
        if capability_flags.contains(CapabilityFlags::CLIENT_SECURE_CONNECTION) {
            let additional_data_len = max(12, auth_data_len - 9);
            let additional_data = reader.read_bytes(additional_data_len as usize)?;
            auth_plugin_data.extend_from_slice(additional_data.as_slice());
            reader.read_u8()?;
        }
        let auth_plugin_name = reader.read_string(true)?;
        Ok(Handshake {
            protocol_version,
            server_version,
            connection_id,
            auth_plugin_data,
            capability_flags,
            character_set,
            status_flags,
            auth_plugin_name
        })
    }

    fn write(&self, buf: &mut impl bytes::BufMut, capabilities: CapabilityFlags) -> Result<(), Error> {
        Err(Error::new(
            ErrorKind::NotSupported,
            "writing handshake packets"))

    }
}

pub struct HandshakeResponse {
    pub capability_flags: CapabilityFlags,
    pub max_packet_size: u32,
    pub character_set: u8,
    pub username: String,
    pub auth_response: Vec<u8>,
    pub initial_database: Option<String>,
    pub auth_plugin_name: String,
    pub attributes: Vec<(String, String)>
}

impl Packet for HandshakeResponse {
    fn write(&self, buf: &mut impl BufMut, _: CapabilityFlags) -> Result<(), Error> {
        let mut writer = buf.writer();

        let cap_flags = if let Some(_) = self.initial_database {
            self.capability_flags | CapabilityFlags::CLIENT_CONNECT_WITH_DB
        } else {
            self.capability_flags
        };

        writer.write_u32::<LE>(cap_flags.bits())?;
        writer.write_u32::<LE>(self.max_packet_size)?;
        writer.write_u8(self.character_set)?;
        writer.write_all(&[0u8; 23])?;
        writer.write_string(self.username.as_ref())?;

        if self.capability_flags.contains(CapabilityFlags::CLIENT_PLUGIN_AUTH_LENENC_CLIENT_DATA) {
            writer.write_lenenc_bytes(&self.auth_response)?;
        } else if self.capability_flags.contains(CapabilityFlags::CLIENT_SECURE_CONNECTION) {
            if self.auth_response.len() > u8::MAX as usize {
                return Err(Error::new(
                    ErrorKind::InvalidPacket,
                    format!("cannot encode auth response of length {} unless CLIENT_PLUGIN_AUTHN_LENENC_CLIENT_DATA is set", self.auth_response.len())))
            }
            writer.write_u8(self.auth_response.len() as u8)?;
            writer.write_all(&self.auth_response)?;
        } else {
            writer.write_all(&self.auth_response)?;
            writer.write_u8(0)?;
        }

        if let Some(s) = &self.initial_database {
            writer.write_string(s.as_ref())?;
        }

        if cap_flags.contains(CapabilityFlags::CLIENT_PLUGIN_AUTH) {
            writer.write_string(self.auth_plugin_name.as_ref())?;
        }

        if cap_flags.contains(CapabilityFlags::CLIENT_CONNECT_ATTRS) {
            writer.write_lenenc_int(self.attributes.len() as u64)?;
            for (key, val) in &self.attributes {
                writer.write_lenenc_string(key)?;
                writer.write_lenenc_string(val)?;
            }
        }
        Ok(())
    }

    fn read(_: &mut impl Buf, _: CapabilityFlags) -> Result<Self, Error> {
        Err(Error::new(
            ErrorKind::NotSupported,
            "reading handshake response packets"))
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use mysql_common::constants::{CapabilityFlags, StatusFlags};
    use crate::packet::Packet;
    use super::{Handshake, HandshakeResponse};

    const HANDSHAKE_PACKET: [u8; 74] = [
        0x0Au8, 0x38u8, 0x2Eu8, 0x30u8, 0x2Eu8, 0x32u8, 0x32u8, 0x00u8, 0x0Au8, 0x00u8, 0x00u8, 0x00u8, 0x26u8, 0x43u8, 0x30u8,
        0x04u8, 0x76u8, 0x14u8, 0x45u8, 0x0Du8, 0x00u8, 0xFFu8, 0xFFu8, 0xFFu8, 0x02u8, 0x00u8, 0xFFu8, 0xC7u8, 0x15u8, 0x00u8,
        0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x6Fu8, 0x70u8, 0x35u8, 0x1Du8, 0x38u8, 0x30u8,
        0x7Eu8, 0x3Fu8, 0x23u8, 0x05u8, 0x60u8, 0x5Fu8, 0x00u8, 0x63u8, 0x61u8, 0x63u8, 0x68u8, 0x69u8, 0x6Eu8, 0x67u8, 0x5Fu8,
        0x73u8, 0x68u8, 0x61u8, 0x32u8, 0x5Fu8, 0x70u8, 0x61u8, 0x73u8, 0x73u8, 0x77u8, 0x6Fu8, 0x72u8, 0x64u8, 0x00u8];

    #[test]
    pub fn parse_can_parse_handshake_packets() {
        let handshake = Handshake::read(
            &mut Bytes::from_static(&HANDSHAKE_PACKET),
            CapabilityFlags::from_bits_truncate(0)).unwrap();
        let expected_flags =
            CapabilityFlags::CLIENT_LONG_PASSWORD | CapabilityFlags::CLIENT_FOUND_ROWS | CapabilityFlags::CLIENT_LONG_FLAG |
            CapabilityFlags::CLIENT_CONNECT_WITH_DB | CapabilityFlags::CLIENT_NO_SCHEMA | CapabilityFlags::CLIENT_COMPRESS |
            CapabilityFlags::CLIENT_ODBC | CapabilityFlags::CLIENT_LOCAL_FILES | CapabilityFlags::CLIENT_IGNORE_SPACE |
            CapabilityFlags::CLIENT_PROTOCOL_41 | CapabilityFlags::CLIENT_INTERACTIVE | CapabilityFlags::CLIENT_SSL |
            CapabilityFlags::CLIENT_IGNORE_SIGPIPE | CapabilityFlags::CLIENT_TRANSACTIONS | CapabilityFlags::CLIENT_RESERVED |
            CapabilityFlags::CLIENT_SECURE_CONNECTION | CapabilityFlags::CLIENT_MULTI_STATEMENTS | CapabilityFlags::CLIENT_MULTI_RESULTS |
            CapabilityFlags::CLIENT_PS_MULTI_RESULTS | CapabilityFlags::CLIENT_PLUGIN_AUTH | CapabilityFlags::CLIENT_CONNECT_ATTRS |
            CapabilityFlags::CLIENT_PLUGIN_AUTH_LENENC_CLIENT_DATA | CapabilityFlags::CLIENT_CAN_HANDLE_EXPIRED_PASSWORDS | CapabilityFlags::CLIENT_SESSION_TRACK |
            CapabilityFlags::CLIENT_DEPRECATE_EOF | CapabilityFlags::CLIENT_SSL_VERIFY_SERVER_CERT | CapabilityFlags::CLIENT_REMEMBER_OPTIONS;
        assert_eq!(10, handshake.protocol_version);
        assert_eq!("8.0.22", handshake.server_version);
        assert_eq!(10, handshake.connection_id);
        assert_eq!(vec![38u8, 67, 48, 4, 118, 20, 69, 13, 111, 112, 53, 29, 56, 48, 126, 63, 35, 5, 96, 95], handshake.auth_plugin_data);
        assert_eq!(expected_flags, handshake.capability_flags);
        assert_eq!(255, handshake.character_set);
        assert_eq!(StatusFlags::SERVER_STATUS_AUTOCOMMIT, handshake.status_flags);
        assert_eq!("caching_sha2_password", handshake.auth_plugin_name);
    }

    #[test]
    pub fn write_handshake_response() {
        let base_flags = CapabilityFlags::from_bits_truncate(0x81bea205);
        let expected: Vec<u8> = vec![
            0x05, 0xa2, 0xbe, 0x81, // client capabilities
            0x00, 0x00, 0x00, 0x01, // max packet
            0x2d, // charset
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // reserved
            b'r', b'o', b'o', b't', 0x00, // username=root
            0x00, // blank scramble
            b'm', b'y', b's', b'q', b'l', b'_',
            b'n', b'a', b't', b'i', b'v', b'e', b'_',
            b'p', b'a', b's', b's', b'w', b'o', b'r', b'd', 0x00, // auth plugin name
            0x00,
        ];
        assert_eq!(expected, write_to_vec(HandshakeResponse {
            capability_flags: base_flags,
            max_packet_size: 0x1000000,
            character_set: 0x2d,
            username: "root".into(),
            auth_response: vec![],
            auth_plugin_name: "mysql_native_password".into(),
            attributes: vec![],
            initial_database: None
        }));

        // Include a db name but don't set the caps for it -> we set it for you
        let expected: Vec<u8> = vec![
            0x0d, 0xa2, 0xbe, 0x81, // client capabilities
            0x00, 0x00, 0x00, 0x01, // max packet
            0x2d, // charset
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // reserved
            b'r', b'o', b'o', b't', 0x00, // username=root
            0x00, // blank scramble
            b'm', b'y', b'd', b'b', 0x00, // dbname
            b'm', b'y', b's', b'q', b'l', b'_',
            b'n', b'a', b't', b'i', b'v', b'e', b'_',
            b'p', b'a', b's', b's', b'w', b'o', b'r', b'd', 0x00, // auth plugin name
            0x00, // no attrs
        ];
        assert_eq!(expected, write_to_vec(HandshakeResponse {
            capability_flags: base_flags,
            max_packet_size: 0x1000000,
            character_set: 0x2d,
            username: "root".into(),
            auth_response: vec![],
            auth_plugin_name: "mysql_native_password".into(),
            attributes: vec![],
            initial_database: Some("mydb".into()),
        }));
    }

    fn write_to_vec<P: Packet>(packet: P) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::new();
        packet.write(&mut v, CapabilityFlags::from_bits_truncate(0)).unwrap();
        v
    }
}
