use reqwest;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Reqwest(err: reqwest::Error) {
            description(err.description())
                from()
        }
    }
}

#[derive(Debug, Deserialize)]
struct LedgerInfo {
    currency_code: String,
    // TODO can scale be negative?
    currency_scale: u32,
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
    let spspDetails: SpspReceiver = resp.json()?;
    Ok(spspDetails)
}

pub fn quote_source(receiver: &str, source_amount: u64) -> Result<u64, Error> {
    let spspDetails = query(receiver)?;
    println!("{:?}", spspDetails);
    Ok(10)
}

pub fn quote_destination(receiver: &str, destination_amount: u64) -> Result<u64, Error> {
    Ok(10)
}

pub fn pay(receiver: &str, source_amount: u64, destination_amount: u64) -> () {
    println!("Send payment to {} with source amount: {} and destination amount: {}", receiver, source_amount, destination_amount);
}
