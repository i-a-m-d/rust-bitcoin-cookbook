# The `Network` Type
Bitcoin consists of a peer-to-peer (P2P) network of nodes running some version of the Bitcoin software. Over this network, nodes exchange messages with network peers for a number of purposes, like propagating transactions and blocks. The production network they do this on is called Mainnet. For testing purposes however, other variants of the network exist, like Testnet3, Testnet4, Signet and Regtest. 

The network choice is not just a label. It impacts key components of the Bitcoin system, like:
- the address formats used to send and receive coins
- the magic bytes used for p2p messages
- the blockchain the network records its transactions on

The `Network` type represents these different Bitcoin networks. It is an enum with the following variants:
- `Network::Bitcoin`
- `Network::Testnet`
- `Network::Testnet4`
- `Network::Signet`
- `Network::Regtest`

This section gives some practical examples of how and where to use the `Network` type.