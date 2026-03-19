# The `Network` Type

`bitcoin::Network` is an enum that allows you to select which Bitcoin network and blockchain your code is targeting. In practice, it is one of the first types you touch because it affects things like address encoding, parsing and chain-specific constants.

In `bitcoin 0.32.8`, the variants are:

- `Network::Bitcoin`
- `Network::Testnet`
- `Network::Testnet4`
- `Network::Signet`
- `Network::Regtest`
