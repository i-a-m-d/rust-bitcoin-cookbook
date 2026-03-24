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

## Generate a P2WPKH address on different networks

A concrete use case of the `Network` type is during address construction. Different prefixes are used for different networks, leading to different address strings. For P2WPKH address for exampole, this results in the address starting with:
- `bc1` on `Bitcoin`
- `tb1` on `Testnet`, `Testnet4`, and `Signet`
- `bcrt1` on `Regtest`

```rust
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::{Address, CompressedPublicKey, Network};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&[1u8; 32])?;
    let compressed = CompressedPublicKey(secret_key.public_key(&secp));

    let mainnet = Address::p2wpkh(&compressed, Network::Bitcoin);
    let testnet = Address::p2wpkh(&compressed, Network::Testnet);
    let testnet4 = Address::p2wpkh(&compressed, Network::Testnet4);
    let signet = Address::p2wpkh(&compressed, Network::Signet);
    let regtest = Address::p2wpkh(&compressed, Network::Regtest);

    println!("mainnet:  {}", mainnet);
    println!("testnet:  {}", testnet);
    println!("testnet4: {}", testnet4);
    println!("signet:   {}", signet);
    println!("regtest:  {}", regtest);

    Ok(())
}
```