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