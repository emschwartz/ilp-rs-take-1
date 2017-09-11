extern crate clap;
extern crate reqwest;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate quick_error;
extern crate ilp_packet;

use clap::{App, SubCommand, Arg};

mod spsp;
mod ilqp;

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
                let source_amount: f64 = matches.value_of("source_amount").unwrap().parse().unwrap();
                let destination_amount = spsp::quote_source(receiver, source_amount);
                println!("{}", destination_amount.unwrap())
            } else {
                let destination_amount: f64 = matches.value_of("destination_amount").unwrap().parse().unwrap();
                let source_amount = spsp::quote_destination(receiver, destination_amount);
                println!("{}", source_amount.unwrap())
            }
        },
        Some("pay") => {
            let matches = matches.subcommand_matches("pay").unwrap();
            let receiver = matches.value_of("receiver").unwrap();
            let source_amount: f64 = matches.value_of("source_amount").unwrap().parse().unwrap();
            let destination_amount: f64 = matches.value_of("destination_amount").unwrap().parse().unwrap();
            spsp::pay(receiver, source_amount, destination_amount)
        },
        Some(command) => println!("unknown command: {}", command),
        None => println!("command is required")
    }
}
