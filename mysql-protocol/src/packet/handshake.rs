use std::cmp::max;

use mysql_common::constants::CapabilityFlags;

use crate::error::Error;

use super::{PacketReader, read_bytes, read_string, read_u16, read_u32, read_u8};

#[derive(Debug)]
pub struct Handshake {
    protocol_version: u8,
    server_version: String,
    connection_id: u32,
    auth_plugin_data: Vec<u8>,
    capability_flags: CapabilityFlags,
    character_set: Option<u8>,
    status_flags: Option<u16>,
    auth_plugin_name: String
}

impl PacketReader for Handshake {
    type Packet = Handshake;

    fn parse(mut payload: &[u8]) -> Result<Handshake, Error> {
        let protocol_version = read_u8(&mut payload)?;
        let server_version = read_string(&mut payload, false)?;
        let connection_id = read_u32(&mut payload)?;
        let mut auth_plugin_data: Vec<_> = read_bytes(&mut payload, 8)?.into();
        // skip the filler
        payload = &payload[1..];
        let mut capability_flags = CapabilityFlags::from_bits_truncate(read_u16(&mut payload)? as u32);
        let mut character_set = None;
        let mut status_flags = None;
        let mut auth_plugin_name = String::new();
        if payload.len() > 0 {
            character_set = Some(read_u8(&mut payload)?);
            status_flags = Some(read_u16(&mut payload)?);
            capability_flags |= CapabilityFlags::from_bits_truncate((read_u16(&mut payload)? as u32) << 16);
            let auth_data_len = read_u8(&mut payload)?;
            payload = &payload[10..];
            if capability_flags.contains(CapabilityFlags::CLIENT_SECURE_CONNECTION) {
                let additional_data_len = max(12, auth_data_len - 9);
                let additional_data = read_bytes(&mut payload, additional_data_len as usize)?;
                payload = &payload[1..];
                auth_plugin_data.extend_from_slice(additional_data);
            }
            auth_plugin_name = read_string(&mut payload, true)?;
        }
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
}
