quick_error! {
    #[derive(Debug)]
    pub enum Error {

    }
}

pub fn quote_source(destination_account: &str, source_amount: u64) -> Result<u64, Error> {
    // TODO return Result
    Ok(0)
}

pub fn quote_destination(destination_account: &str, destination_amount: u64) -> Result<u64, Error> {
    // TODO return Result
    Ok(0)
}
