use reqwest;
use ilqp;
use psk;
use base64;
use plugin;
use serde_json;
use uuid::{Uuid, UuidVersion};
use chrono::prelude::*;
use chrono::Duration;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Reqwest(err: reqwest::Error) {
            description(err.description())
            from()
        }
        Ilqp(err: ilqp::Error) {
            description(err.description())
            from()
        }
    }
}

#[derive(Debug, Deserialize)]
struct LedgerInfo {
    currency_code: String,
    // TODO can scale be negative?
    currency_scale: i32,
}

    #[derive(Debug, Deserialize)]
struct ReceiverInfo {
    name: String,
    image_url: String,
    identifier: String,
}

#[derive(Debug, Deserialize)]
struct SpspReceiver {
    destination_account: String,
    shared_secret: String,
    maximum_destination_amount: String,
    minimum_destination_amount: String,
    ledger_info: LedgerInfo,
    receiver_info: ReceiverInfo,
}

fn query(receiver: &str) -> Result<SpspReceiver, Error> {
    // TODO actually use webfinger
    let resp = &mut reqwest::get(receiver)?;
    // TODO what if the response doesn't match?
    let spsp_details: SpspReceiver = resp.json()?;
    Ok(spsp_details)
}

fn float_to_int(amount: f64, scale: i32) -> u64 {
    (amount * 10.0_f64.powi(scale)).floor() as u64
}

fn int_to_float(amount: u64, scale: i32) -> f64 {
    amount as f64 * 10.0_f64.powi(0 - scale)
}

pub fn quote_source(receiver: &str, source_amount: f64) -> Result<f64, Error> {
    let spsp_details = query(receiver)?;
    let destination_account = spsp_details.destination_account;
    // TODO shift by scale from ledger plugin
    let source_scale = 1;
    let source_amount = float_to_int(source_amount, source_scale);
    let destination_hold_duration = 10000;
    let destination_amount = ilqp::quote_source(&destination_account, source_amount, destination_hold_duration)?;
    let destination_amount = int_to_float(destination_amount, spsp_details.ledger_info.currency_scale);
    Ok(destination_amount)
}

pub fn quote_destination(receiver: &str, destination_amount: f64) -> Result<f64, Error> {
    let spsp_details = query(receiver)?;
    let destination_account = spsp_details.destination_account;
    let destination_amount = float_to_int(destination_amount, spsp_details.ledger_info.currency_scale);
    let destination_hold_duration = 10000;
    let source_amount = ilqp::quote_destination(&destination_account, destination_amount, destination_hold_duration)?;
    // TODO shift by scale from ledger plugin
    let source_scale = 1;
    Ok(int_to_float(source_amount, source_scale))
}

pub fn pay(receiver: &str, source_amount: f64, destination_amount: f64) -> Result<(), Error> {
    println!("Send payment to {} with source amount {} and destination amount {}", receiver, source_amount, destination_amount);
    let spsp_details = query(receiver)?;
    println!("Got receiver details: {:?}", spsp_details);
    let shared_secret = base64::decode_config(&spsp_details.shared_secret, base64::URL_SAFE_NO_PAD).unwrap();
    let destination_amount = float_to_int(destination_amount, spsp_details.ledger_info.currency_scale);
    let (packet, condition) = psk::create_packet_and_condition(
        &shared_secret,
        &spsp_details.destination_account,
        destination_amount);
    println!("Created packet: {:?} and condition: {:?}", packet, condition);

    // TODO get scale from plugin
    let source_scale = 6;
    let transfer = plugin::Transfer {
        id: Uuid::new(UuidVersion::Random).unwrap().hyphenated().to_string(),
        from: "".to_string(),
        to: spsp_details.destination_account.to_string(),
        ledger: "".to_string(),
        amount: float_to_int(source_amount, source_scale),
        ilp: packet,
        execution_condition: condition,
        expires_at: (Utc::now().checked_add_signed(Duration::seconds(60)).unwrap()).to_rfc3339()
    };
    println!("Sending transfer: {}", serde_json::to_string(&transfer).unwrap());

    Ok(())
}
