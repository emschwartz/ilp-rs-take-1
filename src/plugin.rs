use serde::{Serializer, Deserialize, Deserializer};
use base64;
use ilp_packet::oer;

// TODO turn plugin interface into trait

quick_error! {
    #[derive(Debug)]
    pub enum Error {

    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
    pub id: String,
    pub from: String,
    pub to: String,
    pub ledger: String,
    pub amount: u64,
    #[serde(serialize_with = "as_base64")]
    pub ilp: Vec<u8>,
    #[serde(serialize_with = "as_base64")]
    pub execution_condition: Vec<u8>,
    pub expires_at: String,
}

pub fn as_base64<T, S>(buffer: &T, serializer: S) -> Result<S::Ok, S::Error>
  where T: AsRef<[u8]>,
        S: Serializer
{
    serializer.serialize_str(&base64::encode_config(buffer.as_ref(), base64::URL_SAFE_NO_PAD))
}

pub struct Plugin {

}

impl Plugin {
    pub fn connect() -> () {

    }

    pub fn send_transfer(transfer: Transfer) -> Result<(), Error> {

        Ok(())
    }

}
