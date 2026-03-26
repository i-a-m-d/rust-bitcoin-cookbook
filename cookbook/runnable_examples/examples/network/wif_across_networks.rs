use bitcoin::secp256k1::SecretKey;
use bitcoin::{Network, PrivateKey};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let secret_key = SecretKey::from_slice(&[1u8; 32])?;

    let mainnet = PrivateKey::new(secret_key, Network::Bitcoin);
    let testnet = PrivateKey::new(secret_key, Network::Testnet);
    let signet = PrivateKey::new(secret_key, Network::Signet);
    let regtest = PrivateKey::new(secret_key, Network::Regtest);

    println!("{:<7}: {}", "mainnet", mainnet.to_wif());
    println!("{:<7}: {}", "testnet", testnet.to_wif());
    println!("{:<7}: {}", "signet", signet.to_wif());
    println!("{:<7}: {}", "regtest", regtest.to_wif());

    Ok(())
}