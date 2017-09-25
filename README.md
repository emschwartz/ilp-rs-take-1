# Rust ILP Client
> For sending [Interledger](https://interledger.org) payments

## Installation

**TODO**

## CLI Usage

**TODO**

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
- [ ] Switch log statements to debug_env
- [ ] Add incoming event stream that parses messages
- [ ] Add async prepare function (that doesn't wait for the fulfill)
- [ ] Implement ILQP
- [ ] Add support for memos in PSK and SPSP
- [ ] Refactor ILP, PSK, etc into separate modules and export as library
- [ ] Add support for receiving payments
- [ ] Finish plugin interface and make it a trait
- [ ] Implement other plugins
