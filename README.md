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
- [ ] Properly return fulfillments from plugin
- [ ] Clean up event handling mechanism in Plugin (either by passing in closures to event listening functions or by making the plugin a trait that users implement with their own event handlers)
- [ ] Implement ILQP
- [ ] Add support for memos in PSK and SPSP
- [ ] Refactor ILP, PSK, etc into separate modules
- [ ] Add support for receiving payments
- [ ] Implement other plugins
