use bincode::{deserialize, serialize};
use bitcoincash_addr::{Address, HashType, Scheme};
use ed25519_dalek::SigningKey;
use rand::{rngs::OsRng, RngCore};
use ripemd::{Digest, Ripemd160};
use serde::{Deserialize, Serialize};
use sha256;
use sled;

use super::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub secret_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl Wallet {
    fn new() -> Self {
        let mut csprng = OsRng;
        let ed25519_key = SigningKey::generate(&mut csprng);
        let secret_key = ed25519_key.to_bytes().to_vec();
        let public_key = ed25519_key.verifying_key().to_bytes().to_vec();
        Wallet {
            secret_key,
            public_key,
        }
    }

    fn get_address(&self) -> String {
        let mut public_key = self.public_key.clone();
        let pub_key_hash = hash_pub_key(&mut public_key);

        let address = Address {
            body: pub_key_hash,
            scheme: Scheme::Base58,
            hash_type: HashType::Script,
            ..Default::default()
        };
        address.encode().unwrap()
    }
}

pub fn hash_pub_key(pub_key: &Vec<u8>) -> Vec<u8> {
    let pub_key = sha256::digest(pub_key);
    let mut pub_key_hasher = Ripemd160::new();
    pub_key_hasher.update(pub_key);

    pub_key_hasher.finalize().to_vec()
}

pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

impl Wallets {
    pub fn new() -> Result<Wallets> {
        let mut wlts = Wallets {
            wallets: HashMap::<String, Wallet>::new(),
        };

        let db = sled::open("./db/wallets")?;

        for item in db.into_iter() {
            let i = item?;
            let address = String::from_utf8(i.0.to_vec())?;
            let wallet = deserialize(&i.1.to_vec())?;
            wlts.wallets.insert(address, wallet);
        }

        Ok(wlts)
    }

    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.get_address();
        self.wallets.insert(address.clone(), wallet);
        info!("create wallet: {}", address);
        address
    }

    pub fn get_all_addresses(&self) -> Vec<String> {
        let mut addresses: Vec<String> = Vec::new();

        for (address, _) in &self.wallets {
            addresses.push(address.clone());
        }

        addresses
    }

    pub fn save_all(&self) -> Result<()> {
        let db = sled::open("./db/wallets")?;

        for (address, wallet) in &self.wallets {
            let wallet_data = serialize(wallet)?;
            db.insert(address, wallet_data);
        }
        db.flush()?;

        Ok(())
    }

    // --------- getter ---------

    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
        self.wallets.get(address)
    }
}
