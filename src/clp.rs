use std;
use std::string;
use std::io::{Read, Write};
use ilp_packet::oer;
use std::io::{Cursor};
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use chrono;
use chrono::{DateTime, Utc, TimeZone};

const DATE_TIME_FORMAT: &'static str = "%Y%m%d%H%M%S%.3fZ";

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        UnknownPacket(descr: &'static str) {
            description(descr)
        }
        Io(err: std::io::Error) {
            description(err.description())
            from()
        }
        Chrono(err: chrono::ParseError) {
            description(err.description())
            from()
        }
        Utf8(err: std::string::FromUtf8Error) {
            description(err.description())
            from()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
#[repr(u8)]
pub enum ContentType {
    ApplicationOctetString = 0,
    TextPlainUtf8 = 1,
    ApplicationJson = 2,
    Unknown,
}

impl From<u8> for ContentType {
    fn from(type_int: u8) -> Self {
        match type_int {
            0 => ContentType::ApplicationOctetString,
            1 => ContentType::TextPlainUtf8,
            2 => ContentType::ApplicationJson,
            _ => ContentType::Unknown,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
#[repr(u8)]
pub enum PacketType {
    //Ack = 1,
    //Response = 2,
    //Error = 3,
    Prepare = 4,
    //Fulfill = 5,
    //Reject = 6,
    //Message = 7,
    Unknown,
}

impl From<u8> for PacketType {
    fn from(type_int: u8) -> Self {
        match type_int {
            //1 => PacketType::Ack,
            //2 => PacketType::Response,
            //3 => PacketType::Error,
            4 => PacketType::Prepare,
            //5 => PacketType::Fulfill,
            //6 => PacketType::Reject,
            //7 => PacketType::Message,
            _ => PacketType::Unknown
        }
    }
}

trait Serializable<T> {
    fn from_bytes(bytes: &[u8]) -> Result<T, Error>;
    fn to_bytes(&self) -> Result<Vec<u8>, Error>;
}

pub struct ProtocolData {
    protocol_name: String,
    content_type: ContentType,
    data: Vec<u8>,
}

pub enum PacketContents {
    Prepare(Prepare),
}

pub struct ClpPacket {
    packet_type: PacketType,
    request_id: u32,
    data: PacketContents,
}

impl Serializable<ClpPacket> for ClpPacket {
    fn from_bytes(bytes: &[u8]) -> Result<ClpPacket, Error> {
        let mut reader = Cursor::new(bytes);
        let packet_type = PacketType::from(reader.read_u8()?);
        let request_id = reader.read_u32::<BigEndian>()?;
        // TODO use read_to_end
        let content_bytes = oer::read_var_octet_string(&bytes[reader.position() as usize..])?;
        let data: PacketContents = match packet_type {
            //PacketType::Ack => Ack::from_bytes(content_bytes)?,
            //PacketType::Response => Response::from_bytes(content_bytes)?,
            //PacketType::Error => Error::from_bytes(content_bytes)?,
            PacketType::Prepare => PacketContents::Prepare(Prepare::from_bytes(content_bytes)?),
            //PacketType::Fulfill => Fulfill::from_bytes(content_bytes)?,
            //PacketType::Reject => Reject::from_bytes(content_bytes)?,
            //PacketType::Message => Message::from_bytes(content_bytes)?,
            PacketType::Unknown => return Err(Error::UnknownPacket("packet type unknown")),
        };
        Ok(ClpPacket {
            packet_type,
            request_id,
            data,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.write_u8(self.packet_type.clone() as u8)?;
        bytes.write_u32::<BigEndian>(self.request_id)?;
        let content_bytes = match self.data {
            PacketContents::Prepare(ref contents) => contents,
        }.to_bytes()?;
        oer::write_var_octet_string(&mut bytes, &content_bytes)?;
        Ok(bytes)
    }
}

pub struct Prepare {
    transfer_id: [u8; 16],
    amount: u64,
    execution_condition: [u8; 32],
    expires_at: DateTime<Utc>,
    protocol_data: Vec<ProtocolData>,
}

impl Serializable<Prepare> for Prepare {
    fn from_bytes(bytes: &[u8]) -> Result<Prepare, Error> {
        let mut reader = Cursor::new(bytes);
        let mut transfer_id = [0u8; 16];
        reader.read_exact(&mut transfer_id)?;
        let amount = reader.read_u64::<BigEndian>()?;
        let mut execution_condition = [0u8; 32];
        reader.read_exact(&mut execution_condition)?;
        let expires_at = String::from_utf8(oer::read_var_octet_string(&bytes[reader.position() as usize..])?.to_vec())?;
        let expires_at = DateTime::parse_from_str(&expires_at, DATE_TIME_FORMAT)?.with_timezone(&Utc);
        // TODO read protocol data
        let protocol_data: Vec<ProtocolData> = Vec::new();
        Ok(Prepare {
            transfer_id,
            amount,
            execution_condition,
            expires_at,
            protocol_data,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.write_all(&self.transfer_id)?;
        bytes.write_u64::<BigEndian>(self.amount)?;
        bytes.write_all(&self.execution_condition)?;
        let expires_at = self.expires_at.naive_utc().format(DATE_TIME_FORMAT).to_string();
        oer::write_var_octet_string(&mut bytes, &expires_at.into_bytes())?;
        // TODO add protocol data
        Ok(bytes)
    }
}

#[cfg(test)]
mod clp_prepare {
    use super::*;

    #[test]
    fn serialize_without_protocol_data() {
        let protocol_data: Vec<ProtocolData> = vec![];
        let prepare1 = ClpPacket {
            packet_type: PacketType::Prepare,
            request_id: 1,
            data: PacketContents::Prepare(Prepare {
                transfer_id: [180,200,56,246,128,177,71,248,168,46,177,252,251,237,137,213],
                amount: 1000,
                execution_condition: [219, 42, 249, 249, 219, 166, 255, 52, 179, 237, 173, 251, 152, 107, 155, 180, 205, 75, 75, 65, 229, 4, 65, 25, 197, 93, 52, 175, 218, 191, 252, 2],
                expires_at: Utc.timestamp(1503919920, 0),
                protocol_data,
            })
        };
        let actual = prepare1.to_bytes().unwrap();
        let expected = vec![4, 0, 0, 0, 1, 129, 143, 180, 200, 56, 246, 128, 177, 71, 248, 168, 46, 177, 252, 251, 237, 137, 213, 0, 0, 0, 0, 0, 0, 3, 232, 219, 42, 249, 249, 219, 166, 255, 52, 179, 237, 173, 251, 152, 107, 155, 180, 205, 75, 75, 65, 229, 4, 65, 25, 197, 93, 52, 175, 218, 191, 252, 2, 19, 50, 48, 49, 55, 48, 56, 50, 56, 48, 57, 51, 50, 48, 48, 46, 48, 48, 48, 90];
        assert_eq!(actual, expected);
    }
}

