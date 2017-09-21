use std;
use std::cmp;
use std::string;
use std::ascii::AsciiExt;
use std::io::{Read, Write};
use ilp_packet::oer::{ReadOerExt, WriteOerExt};
use std::io::{Cursor};
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use chrono;
use chrono::{DateTime, Utc, TimeZone, NaiveDateTime};

const DATE_TIME_FORMAT: &'static str = "%Y%m%d%H%M%S%.3fZ";

// TODO replace these functions with a GeneralizedTime struct
fn datetime_to_bytes(date: DateTime<Utc>) -> Vec<u8> {
    date.naive_utc().format(DATE_TIME_FORMAT).to_string().into_bytes()
}

fn datetime_from_bytes(bytes: Vec<u8>) -> Result<DateTime<Utc>, Error> {
    let date_string = String::from_utf8(bytes)?;
    let utc_date = NaiveDateTime::parse_from_str(&date_string, &DATE_TIME_FORMAT)?;
    let date = DateTime::<Utc>::from_utc(utc_date, Utc);
    Ok(date)
}

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
        Utf8(err: string::FromUtf8Error) {
            description(err.description())
            from()
        }
        Invalid(descr: &'static str) {
            description(descr)
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
#[repr(u8)]
pub enum ContentType {
    ApplicationOctetStream = 0,
    TextPlainUtf8 = 1,
    ApplicationJson = 2,
    Unknown,
}

impl From<u8> for ContentType {
    fn from(type_int: u8) -> Self {
        match type_int {
            0 => ContentType::ApplicationOctetStream,
            1 => ContentType::TextPlainUtf8,
            2 => ContentType::ApplicationJson,
            _ => ContentType::Unknown,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
#[repr(u8)]
pub enum PacketType {
    Response = 1,
    ErrorResponse = 2,
    Prepare = 3,
    Fulfill = 4,
    Reject = 5,
    Message = 6,
    Unknown,
}

impl From<u8> for PacketType {
    fn from(type_int: u8) -> Self {
        match type_int {
            1 => PacketType::Response,
            2 => PacketType::ErrorResponse,
            3 => PacketType::Prepare,
            4 => PacketType::Fulfill,
            5 => PacketType::Reject,
            6 => PacketType::Message,
            _ => PacketType::Unknown
        }
    }
}

pub trait Serializable<T> {
    // TODO rethink whether bytes should be mutable so that the pointer advanced automatically
    fn from_bytes(bytes: &[u8]) -> Result<T, Error>;
    fn to_bytes(&self) -> Result<Vec<u8>, Error>;
}

#[derive(Debug, PartialEq)]
pub struct ProtocolData {
    pub protocol_name: String,
    pub content_type: ContentType,
    pub data: Vec<u8>,
}

impl ProtocolData {
    fn from_bytes_get_length(bytes: &[u8]) -> Result<(ProtocolData, u64), Error> {
        // TODO just read from a mutable reference so we don't need to manually work with the
        // num_bytes_read
        let mut reader = Cursor::new(bytes);
        let protocol_name_bytes = reader.read_var_octet_string()?;
        let protocol_name = String::from_utf8(protocol_name_bytes.to_vec())?;
        println!("protocol_name {}", protocol_name);

        let content_type = ContentType::from(reader.read_u8()?);
        println!("content_type {:?}", content_type);
        let data = reader.read_var_octet_string()?;
        let num_bytes_read = reader.position() as u64;
        Ok((ProtocolData {
            protocol_name,
            content_type,
            data,
        }, num_bytes_read))
    }
}

impl Serializable<ProtocolData> for ProtocolData {
    fn from_bytes(bytes: &[u8]) -> Result<ProtocolData, Error> {
        let (protocol_data, _num_bytes_read) = ProtocolData::from_bytes_get_length(bytes)?;
        Ok(protocol_data)
    }

    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes: Vec<u8> = Vec::new();
        if !self.protocol_name.as_str().is_ascii() {
            return Err(Error::Invalid("protocol_name must be ASCII"))
        }
        bytes.write_var_octet_string(self.protocol_name.as_bytes())?;
        bytes.write_u8(self.content_type.clone() as u8)?;
        bytes.write_var_octet_string(&self.data)?;
        Ok(bytes)
    }
}

fn write_protocol_data(bytes: &mut Vec<u8>, protocol_data: &Vec<ProtocolData>) -> Result<(), Error> {
    let length_prefix = protocol_data.len() as u64;
    // TODO do we need to support more than 255 entries?
    if length_prefix > 255 {
        return Err(Error::Invalid("Does not support more than 255 ProtocolData entries"));
    }
    bytes.write_u8(1)?;
    bytes.write_u8(length_prefix as u8)?;

    for p in protocol_data {
        bytes.write_all(&p.to_bytes()?)?;
    }

    Ok(())
}

fn read_protocol_data(bytes: &[u8]) -> Result<Vec<ProtocolData>, Error> {
    let mut reader = Cursor::new(bytes);
    println!("read protocol data");
    let length_prefix_length_prefix = reader.read_u8()?;
    println!("length prefix length {}", length_prefix_length_prefix);
    let length_prefix = reader.read_uint::<BigEndian>(length_prefix_length_prefix as usize)?;
    let mut data: Vec<ProtocolData> = Vec::new();

    let mut position = reader.position();
    println!("before reading protocol data {:?}", reader);
    for _i in 0..length_prefix {
        let (protocol_data, num_bytes_read) = ProtocolData::from_bytes_get_length(&bytes[position as usize..])?;
        position += num_bytes_read;
        data.push(protocol_data);
    }
    Ok(data)
}

#[derive(Debug, PartialEq)]
pub enum PacketContents {
    Response(Response),
    ErrorResponse(ErrorResponse),
    Prepare(Prepare),
    Reject(Reject),
    Fulfill(Fulfill),
    Message(Message),
}

#[derive(Debug, PartialEq)]
pub struct BtpPacket {
    pub packet_type: PacketType,
    pub request_id: u32,
    pub data: PacketContents,
}

impl Serializable<BtpPacket> for BtpPacket {
    fn from_bytes(bytes: &[u8]) -> Result<BtpPacket, Error> {
        let mut reader = Cursor::new(bytes);
        let packet_type = PacketType::from(reader.read_u8()?);
        let request_id = reader.read_u32::<BigEndian>()?;
        println!("packet type, request id {:?} {:?}", packet_type, request_id);
        // TODO don't copy content_bytes
        let content_bytes = reader.read_var_octet_string()?;
        let data: PacketContents = match packet_type {
            PacketType::Response => PacketContents::Response(Response::from_bytes(&content_bytes)?),
            PacketType::ErrorResponse => PacketContents::ErrorResponse(ErrorResponse::from_bytes(&content_bytes)?),
            PacketType::Prepare => PacketContents::Prepare(Prepare::from_bytes(&content_bytes)?),
            PacketType::Fulfill => PacketContents::Fulfill(Fulfill::from_bytes(&content_bytes)?),
            PacketType::Reject => PacketContents::Reject(Reject::from_bytes(&content_bytes)?),
            PacketType::Message => PacketContents::Message(Message::from_bytes(&content_bytes)?),
            PacketType::Unknown => return Err(Error::UnknownPacket("packet type unknown")),
        };
        Ok(BtpPacket {
            packet_type,
            request_id,
            data,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.write_u8(self.packet_type.clone() as u8)?;
        bytes.write_u32::<BigEndian>(self.request_id)?;
        let content_bytes: Vec<u8> = match self.data {
            PacketContents::Response(ref contents) => contents.to_bytes()?,
            PacketContents::ErrorResponse(ref contents) => contents.to_bytes()?,
            PacketContents::Prepare(ref contents) => contents.to_bytes()?,
            PacketContents::Fulfill(ref contents) => contents.to_bytes()?,
            PacketContents::Reject(ref contents) => contents.to_bytes()?,
            PacketContents::Message(ref contents) => contents.to_bytes()?,
        };
        bytes.write_var_octet_string(&content_bytes)?;
        Ok(bytes)
    }
}

#[derive(Debug, PartialEq)]
pub struct Response {
    pub protocol_data: Vec<ProtocolData>,
}

impl Serializable<Response> for Response {
    fn from_bytes(bytes: &[u8]) -> Result<Response, Error> {
        let protocol_data = read_protocol_data(bytes)?;
        Ok(Response {
            protocol_data,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes: Vec<u8> = Vec::new();
        write_protocol_data(&mut bytes, &self.protocol_data)?;
        Ok(bytes)
    }
}

// Corresponds to BTP Error type
#[derive(Debug, PartialEq)]
pub struct ErrorResponse {
    pub code: String, // 3 ASCII characters
    pub name: String,
    pub triggered_at: DateTime<Utc>,
    pub data: String,
    pub protocol_data: Vec<ProtocolData>,
}

impl Serializable<ErrorResponse> for ErrorResponse {
    fn from_bytes(bytes: &[u8]) -> Result<ErrorResponse, Error> {
        let mut reader = Cursor::new(bytes);
        let mut code = [0u8; 3];
        reader.read_exact(&mut code)?;
        let code = String::from_utf8(code.to_vec())?;
        // TODO name can only be ASCII
        let name = String::from_utf8(reader.read_var_octet_string()?)?;
        let triggered_at_bytes = reader.read_var_octet_string()?;
        let triggered_at = datetime_from_bytes(triggered_at_bytes)?;
        let data = String::from_utf8(reader.read_var_octet_string()?)?;
        let protocol_data_bytes = &bytes[reader.position() as usize..];
        let protocol_data = read_protocol_data(protocol_data_bytes)?;
        Ok(ErrorResponse {
            code,
            name,
            triggered_at,
            data,
            protocol_data,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes: Vec<u8> = Vec::new();
        if self.code.len() as u64 != 3 || !self.code.is_ascii() {
            return Err(Error::Invalid("code must be 3 ASCII characters"));
        }
        bytes.write_all(&self.code.as_bytes()[..3])?;
        bytes.write_var_octet_string(self.name.as_bytes())?;
        bytes.write_var_octet_string(&datetime_to_bytes(self.triggered_at))?;
        bytes.write_var_octet_string(self.data.as_bytes())?;
        write_protocol_data(&mut bytes, &self.protocol_data)?;
        Ok(bytes)
    }
}

#[derive(Debug, PartialEq)]
pub struct Prepare {
    pub transfer_id: [u8; 16],
    pub amount: u64,
    pub execution_condition: [u8; 32],
    pub expires_at: DateTime<Utc>,
    pub protocol_data: Vec<ProtocolData>,
}

impl Serializable<Prepare> for Prepare {
    fn from_bytes(bytes: &[u8]) -> Result<Prepare, Error> {
        let mut reader = Cursor::new(bytes);
        let mut transfer_id = [0u8; 16];
        reader.read_exact(&mut transfer_id)?;
        let amount = reader.read_u64::<BigEndian>()?;
        let mut execution_condition = [0u8; 32];
        reader.read_exact(&mut execution_condition)?;
        let expires_at_bytes = reader.read_var_octet_string()?;
        let expires_at = datetime_from_bytes(expires_at_bytes)?;
        let protocol_data_bytes = &bytes[reader.position() as usize..];
        let protocol_data = read_protocol_data(protocol_data_bytes)?;
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
        let expires_at = datetime_to_bytes(self.expires_at);
        bytes.write_var_octet_string(&expires_at)?;
        write_protocol_data(&mut bytes, &self.protocol_data)?;
        Ok(bytes)
    }
}

#[derive(Debug, PartialEq)]
pub struct Fulfill {
    pub transfer_id: [u8; 16],
    pub fulfillment: [u8; 32],
    pub protocol_data: Vec<ProtocolData>,
}

impl Serializable<Fulfill> for Fulfill {
    fn from_bytes(bytes: &[u8]) -> Result<Fulfill, Error> {
        let mut reader = Cursor::new(bytes);
        let mut transfer_id = [0u8; 16];
        reader.read_exact(&mut transfer_id)?;
        let mut fulfillment = [0u8; 32];
        reader.read_exact(&mut fulfillment)?;
        let protocol_data_bytes = &bytes[reader.position() as usize..];
        let protocol_data = read_protocol_data(protocol_data_bytes)?;
        Ok(Fulfill {
            transfer_id,
            fulfillment,
            protocol_data,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.write_all(&self.transfer_id)?;
        bytes.write_all(&self.fulfillment)?;
        write_protocol_data(&mut bytes, &self.protocol_data)?;
        Ok(bytes)
    }
}

#[derive(Debug, PartialEq)]
pub struct Reject {
    transfer_id: [u8; 16],
    protocol_data: Vec<ProtocolData>,
}

impl Serializable<Reject> for Reject {
    fn from_bytes(bytes: &[u8]) -> Result<Reject, Error> {
        let mut reader = Cursor::new(bytes);
        let mut transfer_id = [0u8; 16];
        reader.read_exact(&mut transfer_id)?;
        let protocol_data_bytes = &bytes[reader.position() as usize..];
        let protocol_data = read_protocol_data(protocol_data_bytes)?;
        Ok(Reject {
            transfer_id,
            protocol_data,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.write_all(&self.transfer_id)?;
        write_protocol_data(&mut bytes, &self.protocol_data)?;
        Ok(bytes)
    }
}

#[derive(Debug, PartialEq)]
pub struct Message {
    protocol_data: Vec<ProtocolData>,
}

impl Serializable<Message> for Message {
    fn from_bytes(bytes: &[u8]) -> Result<Message, Error> {
        let protocol_data = read_protocol_data(bytes)?;
        Ok(Message {
            protocol_data,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes: Vec<u8> = Vec::new();
        write_protocol_data(&mut bytes, &self.protocol_data)?;
        Ok(bytes)
    }
}

#[cfg(test)]
mod generalized_time {
    use super::*;

    #[test]
    fn serialize() {
        let date1 = Utc.timestamp(0, 0);
        let actual1 = datetime_to_bytes(date1);
        let expected1 = [ 49, 57, 55, 48, 48, 49, 48, 49, 48, 48, 48, 48, 48, 48, 46, 48, 48, 48, 90 ];
        assert_eq!(actual1, expected1);

        let date2 = Utc.timestamp(1505444840, 870000000);
        let actual2 = datetime_to_bytes(date2);
        let expected2 = [ 50, 48, 49, 55, 48, 57, 49, 53, 48, 51, 48, 55, 50, 48, 46, 56, 55, 48, 90 ];
        assert_eq!(actual2, expected2);
    }

    #[test]
    fn deserialize() {
        let expected1 = Utc.timestamp(0, 0);
        let actual1 = datetime_from_bytes(vec![ 49, 57, 55, 48, 48, 49, 48, 49, 48, 48, 48, 48, 48, 48, 46, 48, 48, 48, 90 ]).unwrap();
        assert_eq!(actual1, expected1);

        let expected2 = Utc.timestamp(1505444840, 870000000);
        let actual2 = datetime_from_bytes(vec![ 50, 48, 49, 55, 48, 57, 49, 53, 48, 51, 48, 55, 50, 48, 46, 56, 55, 48, 90 ]).unwrap();
        assert_eq!(actual2, expected2);
    }

}

#[cfg(test)]
mod btp_prepare {
    use super::*;

    fn get_instance1() -> BtpPacket {
        BtpPacket {
            packet_type: PacketType::Prepare,
            request_id: 1,
            data: PacketContents::Prepare(Prepare {
                transfer_id: [180,200,56,246,128,177,71,248,168,46,177,252,251,237,137,213],
                amount: 1000,
                execution_condition: [219, 42, 249, 249, 219, 166, 255, 52, 179, 237, 173, 251, 152, 107, 155, 180, 205, 75, 75, 65, 229, 4, 65, 25, 197, 93, 52, 175, 218, 191, 252, 2],
                expires_at: DateTime::parse_from_rfc3339("2017-08-28T09:32:00.000Z").unwrap().with_timezone(&Utc),
                protocol_data: vec![
                    ProtocolData {
                        protocol_name: String::from("ilp"),
                        content_type: ContentType::ApplicationOctetStream,
                        data: vec![1,28,0,0,0,0,0,0,0,100,17,101,120,97,109,112,108,101,46,114,101,100,46,97,108,105,99,101,0,0]
                    },
                    ProtocolData {
                        protocol_name: "foo".to_string(),
                        content_type: ContentType::ApplicationOctetStream,
                        data: b"bar".to_vec()
                    },
                    ProtocolData {
                        protocol_name: "beep".to_string(),
                        content_type: ContentType::TextPlainUtf8,
                        data: b"boop".to_vec()
                    },
                    ProtocolData {
                        protocol_name: "json".to_string(),
                        content_type: ContentType::ApplicationJson,
                        data: b"{}".to_vec()
                    }
                ],
            })
        }
    }

    fn get_bytes1() -> Vec<u8> {
        vec![3, 0, 0, 0, 1, 129, 143, 180, 200, 56, 246, 128, 177, 71, 248, 168, 46, 177, 252, 251, 237, 137, 213, 0, 0, 0, 0, 0, 0, 3, 232, 219, 42, 249, 249, 219, 166, 255, 52, 179, 237, 173, 251, 152, 107, 155, 180, 205, 75, 75, 65, 229, 4, 65, 25, 197, 93, 52, 175, 218, 191, 252, 2, 19, 50, 48, 49, 55, 48, 56, 50, 56, 48, 57, 51, 50, 48, 48, 46, 48, 48, 48, 90, 1, 4, 3, 105, 108, 112, 0, 30, 1, 28, 0, 0, 0, 0, 0, 0, 0, 100, 17, 101, 120, 97, 109, 112, 108, 101, 46, 114, 101, 100, 46, 97, 108, 105, 99, 101, 0, 0, 3, 102, 111, 111, 0, 3, 98, 97, 114, 4, 98, 101, 101, 112, 1, 4, 98, 111, 111, 112, 4, 106, 115, 111, 110, 2, 2, 123, 125]
    }

    #[test]
    fn serialize() {
        assert_eq!(get_instance1().to_bytes().unwrap(), get_bytes1());
    }

    #[test]
    fn deserialize() {
        assert_eq!(BtpPacket::from_bytes(&get_bytes1()).unwrap(), get_instance1());
    }
}

#[cfg(test)]
mod btp_error {
    use super::*;

    fn get_instance1() -> BtpPacket {
        BtpPacket {
            packet_type: PacketType::ErrorResponse,
            request_id: 1,
            data: PacketContents::ErrorResponse(ErrorResponse {
                code: "L13".to_string(),
                name: "errorName".to_string(),
                triggered_at: DateTime::parse_from_rfc3339("2017-08-28T18:32:00.000Z").unwrap().with_timezone(&Utc),
                data: "boo".to_string(),
                protocol_data: vec![
                    ProtocolData {
                        protocol_name: String::from("ilp"),
                        content_type: ContentType::ApplicationOctetStream,
                        data: vec![1,28,0,0,0,0,0,0,0,100,17,101,120,97,109,112,108,101,46,114,101,100,46,97,108,105,99,101,0,0]
                    },
                    ProtocolData {
                        protocol_name: "foo".to_string(),
                        content_type: ContentType::ApplicationOctetStream,
                        data: b"bar".to_vec()
                    },
                    ProtocolData {
                        protocol_name: "beep".to_string(),
                        content_type: ContentType::TextPlainUtf8,
                        data: b"boop".to_vec()
                    },
                    ProtocolData {
                        protocol_name: "json".to_string(),
                        content_type: ContentType::ApplicationJson,
                        data: b"{}".to_vec()
                    }
                ],
            })
        }
    }

    fn get_bytes1() -> Vec<u8> {
        vec![2, 0, 0, 0, 1, 104, 76, 49, 51, 9, 101, 114, 114, 111, 114, 78, 97, 109, 101, 19, 50, 48, 49, 55, 48, 56, 50, 56, 49, 56, 51, 50, 48, 48, 46, 48, 48, 48, 90, 3, 98, 111, 111, 1, 4,3, 105, 108, 112, 0, 30, 1, 28, 0, 0, 0, 0, 0, 0, 0, 100, 17, 101, 120, 97, 109, 112, 108, 101, 46, 114, 101, 100, 46, 97, 108, 105, 99, 101, 0, 0, 3, 102, 111, 111, 0, 3, 98, 97, 114, 4, 98, 101, 101, 112,1, 4, 98, 111, 111, 112, 4, 106, 115, 111, 110, 2, 2, 123, 125]
    }

    #[test]
    fn serialize() {
        assert_eq!(get_instance1().to_bytes().unwrap(), get_bytes1());
    }

    #[test]
    fn deserialize() {
        assert_eq!(BtpPacket::from_bytes(&get_bytes1()).unwrap(), get_instance1());
    }
}

#[cfg(test)]
mod btp_fulfill {
    use super::*;

    fn get_instance1() -> BtpPacket {
        BtpPacket {
            packet_type: PacketType::Fulfill,
            request_id: 1,
            data: PacketContents::Fulfill(Fulfill {
                transfer_id: [180,200,56,246,128,177,71,248,168,46,177,252,251,237,137,213],
                fulfillment: [219, 42, 249, 249, 219, 166, 255, 52, 179, 237, 173, 251, 152, 107, 155, 180, 205, 75, 75, 65, 229, 4, 65, 25, 197, 93, 52, 175, 218, 191, 252, 2],
                protocol_data: vec![
                    ProtocolData {
                        protocol_name: String::from("ilp"),
                        content_type: ContentType::ApplicationOctetStream,
                        data: vec![1,28,0,0,0,0,0,0,0,100,17,101,120,97,109,112,108,101,46,114,101,100,46,97,108,105,99,101,0,0]
                    },
                    ProtocolData {
                        protocol_name: "foo".to_string(),
                        content_type: ContentType::ApplicationOctetStream,
                        data: b"bar".to_vec()
                    },
                    ProtocolData {
                        protocol_name: "beep".to_string(),
                        content_type: ContentType::TextPlainUtf8,
                        data: b"boop".to_vec()
                    },
                    ProtocolData {
                        protocol_name: "json".to_string(),
                        content_type: ContentType::ApplicationJson,
                        data: b"{}".to_vec()
                    }
                ],
            })
        }
    }

    fn get_bytes1() -> Vec<u8> {
        vec![4, 0, 0, 0, 1, 115, 180, 200, 56, 246, 128, 177, 71, 248, 168, 46, 177, 252, 251, 237, 137, 213, 219, 42, 249, 249, 219, 166, 255, 52, 179, 237, 173, 251, 152, 107, 155, 180, 205, 75, 75, 65, 229, 4, 65, 25, 197, 93, 52, 175, 218, 191, 252, 2, 1, 4, 3, 105, 108, 112, 0, 30, 1, 28, 0, 0, 0, 0, 0, 0, 0, 100, 17, 101,120, 97, 109, 112, 108, 101, 46, 114, 101, 100, 46, 97, 108, 105, 99, 101, 0, 0, 3, 102, 111, 111, 0, 3, 98, 97, 114, 4, 98, 101, 101, 112, 1, 4, 98, 111, 111, 112, 4, 106, 115, 111, 110, 2, 2, 123, 125]
    }

    #[test]
    fn serialize() {
        assert_eq!(get_instance1().to_bytes().unwrap(), get_bytes1());
    }

    #[test]
    fn deserialize() {
        assert_eq!(BtpPacket::from_bytes(&get_bytes1()).unwrap(), get_instance1());
    }
}
