use ethsign::SecretKey;
use secp256k1::rand::rngs::OsRng;
use secp256k1::Secp256k1;
use sha3::{Digest, Keccak256};

use std::fs;
use std::path::Path;

pub struct Keys {
    pub secret_key: String,
    pub public_key: String,
}

fn generate_and_save_keys() -> Result<(String, String), Box<dyn std::error::Error>> {
    let secp = Secp256k1::new();
    let mut rng = OsRng::new()?;
    let (secret_key, public_key) = secp.generate_keypair(&mut rng);

    let secret_key_hex = hex::encode(secret_key.secret_bytes());
    let public_key_hex = hex::encode(public_key.serialize_uncompressed());

    // Save keys to files
    fs::write("priv_key.txt", &secret_key_hex)?;
    fs::write("pub_key.txt", &public_key_hex)?;

    Ok((secret_key_hex, public_key_hex))
}

pub fn load_or_generate_keys() -> Result<(String, String), Box<dyn std::error::Error>> {
    if Path::new("priv_key.txt").exists() && Path::new("pub_key.txt").exists() {
        println!("keys loaded");
        let secret_key_hex = fs::read_to_string("priv_key.txt")?;
        let public_key_hex = fs::read_to_string("pub_key.txt")?;
        Ok((
            secret_key_hex.trim().to_string(),
            public_key_hex.trim().to_string(),
        ))
    } else {
        println!("keys generated");
        generate_and_save_keys()
    }
}

pub fn sign_message(
    message: &str,
    private_key_hex: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Convert the hexadecimal private key to bytes
    let private_key_bytes = hex::decode(private_key_hex)?;
    if private_key_bytes.len() != 32 {
        return Err("Private key must be 32 bytes long".into());
    }
    let mut private_key = [0u8; 32];
    private_key.copy_from_slice(&private_key_bytes);

    // Hash the message using Keccak256
    let message_hash = Keccak256::digest(message.as_bytes());

    // Create a SecretKey from the private key
    let secret = SecretKey::from_raw(&private_key)?;

    // Sign the hash
    let signature = secret.sign(&message_hash)?;

    // Format the signature
    let mut sig = [0u8; 65];
    sig[0..32].copy_from_slice(&signature.r);
    sig[32..64].copy_from_slice(&signature.s);
    sig[64] = signature.v + 27; // Convert to Ethereum's v value

    Ok(format!("0x{}", hex::encode(sig)))
}

pub fn public_key_to_address(public_key_hex: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Convert the public key from hex string to bytes
    let public_key_bytes = hex::decode(public_key_hex)?;

    // Hash the public key using keccak256
    let mut hasher = Keccak256::new();
    hasher.update(&public_key_bytes[1..]); // Skip the first byte (prefix)
    let hash = hasher.finalize();

    // Take the last 20 bytes of the hash
    let address = &hash[hash.len() - 20..];

    // Convert the address to a hex string
    Ok(format!("0x{}", hex::encode(address)))
}
