extern crate clap;

use clap::{App, SubCommand, Arg};

fn main() {
    let matches = App::new("spsp")
        .version("0.1.0")
        .author("Evan Schwartz <evan@ripple.com>")
        .about("Command line sending client for ILP/SPSP")
        .subcommand(SubCommand::with_name("quote")
                    .about("Get a quote")
                    .arg(Arg::with_name("source_amount")
                         .takes_value(true)
                         .long("source_amount")
                         .required(true)
                         .conflicts_with("destination_amount"))
                    .arg(Arg::with_name("destination_amount")
                         .takes_value(true)
                         .long("destination_amount")
                         .required(true)
                         .conflicts_with("source_amount"))
                    .arg(Arg::with_name("receiver")
                        .index(1)
                        .required(true)))
        .subcommand(SubCommand::with_name("pay")
                    .about("Send a payment")
                    .arg(Arg::with_name("source_amount")
                         .takes_value(true)
                         .long("source_amount")
                         .required(true))
                    .arg(Arg::with_name("destination_amount")
                         .takes_value(true)
                         .long("destination_amount")
                         .required(true))
                    .arg(Arg::with_name("receiver")
                        .index(1)
                        .required(true)))
        .get_matches();
    match matches.subcommand_name() {
        Some("quote") => {
            let matches = matches.subcommand_matches("quote").unwrap();
            let receiver = matches.value_of("receiver").unwrap();
            if matches.is_present("source_amount") {
                let source_amount = u64::from_str_radix(matches.value_of("source_amount").unwrap(), 10).unwrap();
                let destination_amount = spsp::quote_source(receiver, source_amount);
                println!("{}", destination_amount.unwrap())
            } else {
                let destination_amount = u64::from_str_radix(matches.value_of("destination_amount").unwrap(), 10).unwrap();
                let source_amount = spsp::quote_destination(receiver, destination_amount);
                println!("{}", source_amount.unwrap())
            }
        },
        Some("pay") => {
            let matches = matches.subcommand_matches("pay").unwrap();
            let receiver = matches.value_of("receiver").unwrap();
            let source_amount = u64::from_str_radix(matches.value_of("source_amount").unwrap(), 10).unwrap();
            let destination_amount = u64::from_str_radix(matches.value_of("destination_amount").unwrap(), 10).unwrap();
            spsp::pay(receiver, source_amount, destination_amount)
        },
        Some(command) => println!("unknown command: {}", command),
        None => println!("command is required")
    }
}

extern crate reqwest;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate quick_error;

pub mod spsp {
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
}
