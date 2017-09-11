use ilp_packet;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        ParseError (err: ilp_packet::errors::ParseError) {
            description(err.description())
            from()
        }

    }
}

pub fn quote_source(destination_account: &str, source_amount: u64, destination_hold_duration: u32) -> Result<u64, Error> {
    let request = ilp_packet::packet::IlqpBySourceRequest {
        destination_account: destination_account.to_string(),
        source_amount,
        destination_hold_duration,
    };
    let ilqp_packet = request.to_bytes()?;
    println!("{:?}", ilqp_packet);
    Ok(0)
}

pub fn quote_destination(destination_account: &str, destination_amount: u64, destination_hold_duration: u32) -> Result<u64, Error> {
    let request = ilp_packet::packet::IlqpByDestinationRequest {
        destination_account: destination_account.to_string(),
        destination_amount,
        destination_hold_duration,
    };
    let ilqp_packet = request.to_bytes()?;
    println!("{:?}", ilqp_packet);
    Ok(0)
}
