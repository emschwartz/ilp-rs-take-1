use std::io::{Error as IoError};
use serde::{Serializer, Deserialize, Deserializer};
use base64;
use ilp_packet::oer;
// TODO get rid of duplicate imports
use btp_packet::{BtpPacket, PacketType, ContentType, ProtocolData, PacketContents, Prepare, Fulfill, Serializable, Error as BtpError};
use uuid::Uuid;
use chrono::{DateTime, Utc, ParseError as ChronoError};
use tokio_core::reactor::Core;
use futures::future::Future;
use futures::{Stream, Sink};
use websocket::result::WebSocketError;
use websocket::{ClientBuilder, OwnedMessage, Message};

// TODO turn plugin interface into trait

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: IoError) {
            description(err.description())
            from()
        }
        Ws(err: WebSocketError) {
            description(err.description())
            from()
        }
        NotConnected(method: &'static str) {
            description("Plugin must be connected to call method")
        }
        Serialization(err: BtpError) {
            description(err.description())
            from()
        }
        DateTimeParse(err: ChronoError) {
            description(err.description())
            from()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
    pub id: [u8; 16],
    //pub from: String,
    //pub to: String,
    //pub ledger: String,
    pub amount: u64,
    #[serde(serialize_with = "as_base64")]
    pub ilp: Vec<u8>,
    #[serde(serialize_with = "as_base64")]
    pub execution_condition: [u8; 32],
    pub expires_at: String,
    // TODO add protocol_data
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferFulfillment {
    pub id: [u8; 16],
    #[serde(serialize_with = "as_base64")]
    pub fulfillment: [u8; 32],
    // TODO add protocol_data
}

fn as_base64<T, S>(buffer: &T, serializer: S) -> Result<S::Ok, S::Error>
  where T: AsRef<[u8]>,
        S: Serializer
{
    serializer.serialize_str(&base64::encode_config(buffer.as_ref(), base64::URL_SAFE_NO_PAD))
}

pub struct Plugin {
    server: String,
}

impl Plugin {
    pub fn new(server: String) -> Self {
        Plugin {
            server: server,
        }
    }

    pub fn send_transfer(&mut self, transfer: Transfer) -> Result<(), Error> {
        let packet = BtpPacket {
            packet_type: PacketType::Prepare,
            // TODO use random request_id
            request_id: 1,
            data: PacketContents::Prepare(Prepare {
                transfer_id: transfer.id,
                amount: transfer.amount,
                execution_condition: transfer.execution_condition,
                expires_at: DateTime::parse_from_rfc3339(&transfer.expires_at)?.with_timezone(&Utc),
                protocol_data: vec![
                    ProtocolData {
                        protocol_name: "ilp".to_string(),
                        content_type: ContentType::ApplicationOctetStream,
                        data: transfer.ilp
                    }
                ]
                // TODO add protocol data
            })
        }.to_bytes()?;
        let msg = OwnedMessage::from(Message::binary(packet));
        let mut core = Core::new()?;
        let runner = ClientBuilder::new(&self.server).unwrap()
            .async_connect(None, &core.handle())
            .and_then(|(s, _)| s.send(msg));
        core.run(runner).unwrap();
        Ok(())
    }
}
