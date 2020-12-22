use std::io::Read;

use byteorder::{LittleEndian as LE, ReadBytesExt};
use bytes::{Buf, BufMut};
use mysql_common::{constants::{CapabilityFlags, SessionStateType, StatusFlags}, packets::OkPacket};

use crate::error::{Error, ErrorKind};

use super::{Packet, utils::*};

pub struct OkPacket {
    affected_rows: u64,
    last_insert_id: u64,
    status_flags: StatusFlags,
    warnings: usize,
    info: String,
    session_state_info: Option<(SessionStateType, Vec<u8>)>,
}

impl Packet for OkPacket {
    fn read(buf: &mut impl Buf, capabilities: CapabilityFlags) -> Result<Self, crate::error::Error> {
        let mut reader = buf.reader();
        let header = reader.read_u8()?;
        if header != 0 && header != 0xFE {
            return Err(Error::new(
                ErrorKind::InvalidPacket,
                format!("OK_Packet should start with 0x00 or 0xFE but it started with 0x{:02X}", header)))
        }
        let affected_rows = reader.read_lenenc_int()?;
        let last_insert_id = reader.read_lenenc_int()?;
        let status_flags = if capabilities.contains(CapabilityFlags::CLIENT_PROTOCOL_41) || capabilities.contains(CapabilityFlags::CLIENT_TRANSACTIONS) {
            StatusFlags::from_bits_truncate(reader.read_u16::<LE>()?)
        } else {
            StatusFlags::from_bits_truncate(0)
        };
        let warnings = if capabilities.contains(CapabilityFlags::CLIENT_PROTOCOL_41) {
            reader.read_u16::<LE>()?
        } else {
            0
        } as usize;

        let (info, session_state_info) = if capabilities.contains(CapabilityFlags::CLIENT_SESSION_TRACK) {
            let info = reader.read_lenenc_string()?;
            if status_flags.contains(StatusFlags::SERVER_SESSION_STATE_CHANGED) {
                let session_state_changes = reader.read_lenenc_bytes()?;
                if session_state_changes.len() == 0 {
                    return Err(Error::new(
                        ErrorKind::ProtocolError,
                        "no session state changes, despite flag being set"))
                };
                let typ = match session_state_changes[0] {
                    0x00 => SessionStateType::SESSION_TRACK_SYSTEM_VARIABLES,
                    0x01 => SessionStateType::SESSION_TRACK_SCHEMA,
                    0x02 => SessionStateType::SESSION_TRACK_STATE_CHANGE,
                    0x03 => SessionStateType::SESSION_TRACK_GTIDS,
                    x => return Err(Error::new(
                        ErrorKind::ProtocolError,
                        format!("unknown session state type 0x{:02X}", x)))
                };
                (info, Some((typ, Vec::from(&session_state_changes[1..]))))
            } else {
                (info, None)
            }
        } else {
            (reader.read_string(true)?, None)
        };

        Ok(OkPacket {
            affected_rows,
            last_insert_id,
            status_flags,
            warnings,
            info,
            session_state_info,
        })
    }

    fn write(&self, _: &mut impl BufMut, _capabilities: CapabilityFlags) -> Result<(), crate::error::Error> {
        todo!()
    }
}

pub struct ErrPacket {
    error_code: u16,
    sql_state: Vec<u8>,
    error_message: String
}

impl Packet for ErrPacket {
    fn read(buf: &mut impl Buf, capabilities: CapabilityFlags) -> Result<Self, Error> {
        let mut reader = buf.reader();
        let header = reader.read_u8()?;
        if header != 0xFF {
            return Err(Error::new(
                ErrorKind::InvalidPacket,
                format!("ERR_Packet should start with 0xFF but it started with 0x{:02X}", header)))
        }
        let error_code = reader.read_u16::<LE>()?;
        let sql_state = if capabilities.contains(CapabilityFlags::CLIENT_PROTOCOL_41) {
            reader.read_bytes(6)?
        } else {
            Vec::new()
        };
        let mut error_message = Vec::new();
        reader.read_to_end(&mut error_message)?;
        let error_message = match String::from_utf8(error_message) {
            Ok(s) => s,
            Err(_) => return Err(Error::new(
                ErrorKind::ProtocolError,
                "string is not valid utf-8"))
        };

        Ok(ErrPacket {
            error_code,
            sql_state,
            error_message
        })
    }

    fn write(&self, _w: &mut impl BufMut, _capabilities: CapabilityFlags) -> Result<(), Error> {
        todo!()
    }
}

pub enum OkOrErrPacket {
    OkP(OkPacket),
    ErrP(ErrPacket)
}

impl Packet for OkOrErrPacket {
    fn read(buf: &mut impl Buf, capabilities: CapabilityFlags) -> Result<Self, Error> {
        if buf.remaining() == 0 {
            Err(Error::new(
                ErrorKind::ProtocolError,
                "packet is empty"
            ))
        } else if buf.bytes()[0] == 0x00 || buf.bytes()[0] == 0xFE {
            Ok(OkOrErrPacket::OkP(OkPacket::read(buf, capabilities)?))
        } else if buf.bytes()[0] == 0xFF {
            Ok(OkOrErrPacket::ErrP(ErrPacket::read(buf, capabilities)?))
        } else {
            Err(Error::new(
                ErrorKind::ProtocolError,
                "expected either an OK_Packet or an Err_Packet"))
        }
    }

    fn write(&self, buf: &mut impl BufMut, capabilities: CapabilityFlags) -> Result<(), Error> {
        todo!()
    }
}
