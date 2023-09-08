#![allow(non_snake_case)]

use anyhow::format_err;
use bincode::serialize;
use bitcoincash_addr::Address;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha256;

use super::*;
use crate::wallets::{hash_pub_key, Wallets};
use std::collections::HashMap;

const SUBSIDY: i32 = 10;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TXInput {
    pub txid: String,
    pub vout: i32,
    pub signature: Vec<u8>,
    pub pub_key: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TXOutput {
    pub value: i32,
    pub pub_key_hash: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    pub id: String,
    pub vin: Vec<TXInput>,
    pub vout: Vec<TXOutput>,
}

impl TXInput {
    pub fn use_key(&self, check_pub_key_hash: &[u8]) -> bool {
        let pub_key = self.pub_key.clone();
        let pub_key_hash = hash_pub_key(&pub_key);
        pub_key_hash == check_pub_key_hash
    }
}

impl TXOutput {
    pub fn is_locked_with_key(&self, pub_key_hash: &[u8]) -> bool {
        self.pub_key_hash == pub_key_hash
    }

    pub fn lock(&mut self, address: &str) -> Result<()> {
        let pub_key_hash = Address::decode(address).unwrap().body;
        debug!("lock: {}", address);
        self.pub_key_hash = pub_key_hash;
        Ok(())
    }

    pub fn new(value: i32, address: String) -> Result<Self> {
        let mut tx_output = TXOutput {
            value,
            pub_key_hash: Vec::new(),
        };
        tx_output.lock(&address)?;
        Ok(tx_output)
    }
}

impl Transaction {
    pub fn sign(
        &mut self,
        private_key: &[u8; 32],
        prev_TXs: HashMap<String, Transaction>,
    ) -> Result<()> {
        if self.is_coinbase() {
            return Ok(());
        }

        for vin in &self.vin {
            if prev_TXs.get(&vin.txid).unwrap().id.is_empty() {
                return Err(format_err!("ERROR: Previous transaction is not correct"));
            }
        }

        let mut tx_copy = self.trimmed_copy();

        for in_id in 0..tx_copy.vin.len() {
            let prev_tx = prev_TXs.get(&tx_copy.vin[in_id].txid).unwrap();
            tx_copy.vin[in_id].signature.clear();
            tx_copy.vin[in_id].pub_key = prev_tx.vout[tx_copy.vin[in_id].vout as usize]
                .pub_key_hash
                .clone();
            tx_copy.id = tx_copy.hash()?;
            tx_copy.vin[in_id].pub_key = Vec::new();
            let sign_key: SigningKey = SigningKey::from_bytes(private_key);
            let signature: Signature = sign_key.sign(tx_copy.id.as_bytes());
            self.vin[in_id].signature = signature.to_bytes().to_vec();
        }

        Ok(())
    }

    pub fn hash(&self) -> Result<String> {
        let mut copy = self.clone();
        copy.id = String::new();
        let data = serialize(&copy)?;
        let hash = sha256::digest(data);
        Ok(hash)
    }

    pub fn verify(&self, prev_TXs: HashMap<String, Transaction>) -> Result<bool> {
        if self.is_coinbase() {
            return Ok(true);
        }

        for vin in &self.vin {
            if prev_TXs.get(&vin.txid).unwrap().id.is_empty() {
                return Err(format_err!("ERROR: Previous transaction is not correct"));
            }
        }

        let mut tx_copy = self.trimmed_copy();

        for in_id in 0..tx_copy.vin.len() {
            let prev_tx = prev_TXs.get(&tx_copy.vin[in_id].txid).unwrap();
            tx_copy.vin[in_id].signature.clear();
            tx_copy.vin[in_id].pub_key = prev_tx.vout[tx_copy.vin[in_id].vout as usize]
                .pub_key_hash
                .clone();
            tx_copy.id = tx_copy.hash()?;
            tx_copy.vin[in_id].pub_key = Vec::new();

            let verify_key: VerifyingKey = VerifyingKey::from_bytes(<&[u8; 32]>::try_from(
                self.vin[in_id].pub_key.as_slice(),
            )?)?;

            if verify_key
                .verify(
                    tx_copy.id.as_bytes(),
                    &Signature::from_bytes(<&[u8; 64]>::try_from(
                        self.vin[in_id].signature.as_slice(),
                    )?),
                )
                .is_err()
            {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn trimmed_copy(&self) -> Self {
        let mut vin = Vec::new();
        let mut vout = Vec::new();

        for v in &self.vin {
            vin.push(TXInput {
                txid: v.txid.clone(),
                vout: v.vout.clone(),
                signature: Vec::new(),
                pub_key: Vec::new(),
            })
        }

        for v in &self.vout {
            vout.push(TXOutput {
                value: v.value,
                pub_key_hash: v.pub_key_hash.clone(),
            })
        }

        Transaction {
            id: self.id.clone(),
            vin,
            vout,
        }
    }

    pub fn new_UTXO(from: &str, to: &str, amount: i32, bc: &Blockchain) -> Result<Transaction> {
        info!("new UTXO Transaction from: {} to: {}", from, to);
        let mut vin = Vec::new();

        let wallets = Wallets::new()?;
        let wallet = match wallets.get_wallet(from) {
            Some(wlt) => wlt,
            None => return Err(format_err!("Wallet not found")),
        };

        let pub_key_hash = hash_pub_key(&wallet.public_key);

        let acc_uo = bc.find_spendable_outputs(&pub_key_hash, amount);

        if acc_uo.0 < amount {
            error!("Not Enough balance");
            return Err(format_err!(
                "Not Enough balance: current balance {}",
                acc_uo.0
            ));
        }

        for tx in acc_uo.1 {
            for out in tx.1 {
                let input = TXInput {
                    txid: tx.0.clone(),
                    vout: out,
                    signature: Vec::new(),
                    pub_key: wallet.public_key.clone(),
                };
                vin.push(input);
            }
        }

        let mut vout = vec![TXOutput::new(amount, String::from(to))?];
        if acc_uo.0 > amount {
            vout.push(TXOutput::new(acc_uo.0 - amount, String::from(from))?)
        }

        let mut tx = Transaction {
            id: String::new(),
            vin,
            vout,
        };
        tx.set_id()?;

        bc.sign_transaction(
            &mut tx,
            <&[u8; 32]>::try_from(wallet.secret_key.as_slice())?,
        );

        Ok(tx)
    }

    pub fn new_coinbase(to: String, mut data: String) -> Result<Transaction> {
        info!("new coinbase Transaction to: {}", to);
        if data == String::from("") {
            data += &format!("Reward to {}", to);
        }

        let mut tx = Transaction {
            id: String::new(),
            vin: vec![TXInput {
                txid: String::new(),
                vout: -1,
                signature: Vec::new(),
                pub_key: Vec::from(data.as_bytes()),
            }],
            vout: vec![TXOutput::new(SUBSIDY, to)?],
        };
        tx.set_id()?;
        Ok(tx)
    }

    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.is_empty() && self.vin[0].vout == -1
    }

    pub fn set_id(&mut self) -> Result<()> {
        let hash = sha256::digest(serialize(self)?);
        self.id = hash;
        Ok(())
    }
}
