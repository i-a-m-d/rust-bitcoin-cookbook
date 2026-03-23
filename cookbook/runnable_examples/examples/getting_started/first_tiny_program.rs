use bitcoin::key::Secp256k1;
use bitcoin::{Address, Network, PrivateKey};

fn main() {
    let secp = Secp256k1::new();
    let private_key = PrivateKey::generate(Network::Testnet);
    let public_key = private_key.public_key(&secp);
    let address =
        Address::p2wpkh(&public_key.try_into().expect("compressed key"), Network::Testnet);

    println!("private key (WIF): {}", private_key.to_wif());
    println!("testnet address: {}", address);
}
