# Rust ILP Client
> For sending [Interledger](https://interledger.org) payments

**This project is under heavy development and written by a Rust n00b. Contributors welcome!**

This module is a reimplementation of the [`ilp` Javascript Module](https://github.com/interledgerjs/ilp) in Rust.

## CLI Usage

### Install

```sh

git clone https://github.com/emschwartz/ilp-rs
cd ilp-rs
cargo build --release
```

### Send Payments

(From the `ilp-rs` directory)
```sh
./target/release/ilp pay --source_amount=10 --destination_amount=10 http://localhost:3000
```

## Library Usage

**TODO**

## Roadmap
- [x] Basic CLI for sending SPSP payments
- [x] Basic implementation of SPSP
- [x] Basic implementation of PSK (memos not supported yet)
- [x] Implementation of ILP
- [x] Implementation of BTP for sending transfers and receiving fulfillments
- [x] Properly return fulfillments from plugin
- [ ] Make BTP server configurable through CLI (or config file?)
- [ ] Add incoming event stream that parses messages
- [ ] Add async prepare function (that doesn't wait for the fulfill)
- [ ] Implement ILQP
- [ ] Add support for memos in PSK and SPSP
- [ ] Refactor ILP, PSK, etc into separate modules and export as library
- [ ] Add support for receiving payments
- [ ] Finish plugin interface and make it a trait
- [ ] Implement other plugins
