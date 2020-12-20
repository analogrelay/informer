use std::cmp::max;

use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};
use mysql_common::constants::{CapabilityFlags, StatusFlags};

use crate::error::Error;

use super::{Packet, read_bytes, read_string};

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
    fn read<R: std::io::BufRead>(reader: &mut R) -> Result<Handshake, Error> {
        let protocol_version = reader.read_u8()?;
        let server_version = read_string(reader, false)?;
        let connection_id = reader.read_u32::<LE>()?;
        let mut auth_plugin_data = read_bytes(reader, 8)?;
        // skip the filler
        reader.read_u8()?;
        let mut capability_flags = CapabilityFlags::from_bits_truncate(reader.read_u16::<LE>()? as u32);
        let character_set = reader.read_u8()?;
        let status_flags = StatusFlags::from_bits_truncate(reader.read_u16::<LE>()?);
        capability_flags |= CapabilityFlags::from_bits_truncate((reader.read_u16::<LE>()? as u32) << 16);
        let auth_data_len = reader.read_u8()?;

        read_bytes(reader, 10)?;
        if capability_flags.contains(CapabilityFlags::CLIENT_SECURE_CONNECTION) {
            let additional_data_len = max(12, auth_data_len - 9);
            let additional_data = read_bytes(reader, additional_data_len as usize)?;
            auth_plugin_data.extend_from_slice(additional_data.as_slice());
            reader.read_u8()?;
        }
        let auth_plugin_name = read_string(reader, true)?;
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

    fn write<W: std::io::Write>(&self, w: &mut W) -> Result<(), Error> {
        Err(Error::NotSupported("writing handshake packets is not supported".into()))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use mysql_common::constants::{CapabilityFlags, StatusFlags};
    use crate::packet::Packet;
    use super::Handshake;

    const HANDSHAKE_PACKET: [u8; 74] = [
        0x0Au8, 0x38u8, 0x2Eu8, 0x30u8, 0x2Eu8, 0x32u8, 0x32u8, 0x00u8, 0x0Au8, 0x00u8, 0x00u8, 0x00u8, 0x26u8, 0x43u8, 0x30u8,
        0x04u8, 0x76u8, 0x14u8, 0x45u8, 0x0Du8, 0x00u8, 0xFFu8, 0xFFu8, 0xFFu8, 0x02u8, 0x00u8, 0xFFu8, 0xC7u8, 0x15u8, 0x00u8,
        0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x6Fu8, 0x70u8, 0x35u8, 0x1Du8, 0x38u8, 0x30u8,
        0x7Eu8, 0x3Fu8, 0x23u8, 0x05u8, 0x60u8, 0x5Fu8, 0x00u8, 0x63u8, 0x61u8, 0x63u8, 0x68u8, 0x69u8, 0x6Eu8, 0x67u8, 0x5Fu8,
        0x73u8, 0x68u8, 0x61u8, 0x32u8, 0x5Fu8, 0x70u8, 0x61u8, 0x73u8, 0x73u8, 0x77u8, 0x6Fu8, 0x72u8, 0x64u8, 0x00u8];

    #[test]
    pub fn parse_can_parse_handshake_packets() {
        let handshake = Handshake::read(&mut Cursor::new(&HANDSHAKE_PACKET)).unwrap();
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
}
