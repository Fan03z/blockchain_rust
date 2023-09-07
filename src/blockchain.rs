#![allow(unused)]
#![allow(non_snake_case)]

use anyhow::format_err;
use bincode::{deserialize, serialize};
use sled;

use super::*;
use crate::block::Block;
use crate::transaction::{TXOutput, Transaction};
use std::collections::HashMap;

const GENESIS_COINBASE_DATA: &str =
    "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

#[derive(Debug, Clone)]
pub struct Blockchain {
    tip: String,
    db: sled::Db,
}

pub struct BlockchainIterator<'a> {
    current_hash: String,
    bc: &'a Blockchain,
}

impl Blockchain {
    pub fn new() -> Result<Blockchain> {
        info!("open blockchain");
        let db = sled::open("./db/bc")?;
        let hash = db
            .get("LAST")?
            .expect("Must create a new blockchain database first");
        info!("Found block database");
        let last_hash = String::from_utf8(hash.to_vec())?;
        Ok(Blockchain {
            tip: last_hash.clone(),
            db: db,
        })
    }

    pub fn create_blockchain(address: String) -> Result<Blockchain> {
        info!("Create new blockchain");

        let db = sled::open("./db/bc")?;

        debug!("Creating new block database");

        let cbtx = Transaction::new_coinbase(address, String::from(GENESIS_COINBASE_DATA))?;
        let genesis: Block = Block::new_genesis_block(cbtx);
        db.insert(genesis.get_hash(), serialize(&genesis)?)?;
        db.insert("LAST", genesis.get_hash().as_bytes())?;
        let bc = Blockchain {
            tip: genesis.get_hash(),
            db,
        };
        bc.db.flush()?;
        Ok(bc)
    }

    pub fn iter(&self) -> BlockchainIterator {
        BlockchainIterator {
            current_hash: self.tip.clone(),
            bc: &self,
        }
    }

    pub fn mine_block(&mut self, transactions: Vec<Transaction>) -> Result<()> {
        info!("mine a new block");

        // FIXME: Do not sign and verify transactions temporarily.We will realize it in next stage -- Transaction_Part2.
        // for tx in &transactions {
        //     if !self.verify_transaction(&tx)? {
        //         return Err(format_err!("ERROR: Invalid transaction"));
        //     }
        // }

        let last_hash = self.db.get("LAST")?.expect("Could not get last hash ");
        let new_block = Block::new_block(transactions, String::from_utf8(last_hash.to_vec())?)?;
        self.db
            .insert(new_block.get_hash(), serialize(&new_block)?)?;
        self.db.insert("LAST", new_block.get_hash().as_bytes())?;
        self.db.flush()?;

        self.tip = new_block.get_hash();

        Ok(())
    }

    fn get_prev_TXs(&self, tx: &Transaction) -> Result<HashMap<String, Transaction>> {
        let mut prev_TXs: HashMap<String, Transaction> = HashMap::new();
        for vin in &tx.vin {
            let prev_tx = self.find_transaction(&vin.txid)?;
            prev_TXs.insert(prev_tx.id.clone(), prev_tx);
        }

        Ok(prev_TXs)
    }

    pub fn sign_transaction(&self, tx: &mut Transaction, private_key: &[u8; 32]) -> Result<()> {
        let prev_TXs = self.get_prev_TXs(tx)?;
        tx.sign(private_key, prev_TXs)?;

        Ok(())
    }

    pub fn verify_transaction(&self, tx: &Transaction) -> Result<bool> {
        if tx.is_coinbase() {
            return Ok(true);
        }

        let prev_TXs = self.get_prev_TXs(tx)?;
        tx.verify(prev_TXs)
    }

    pub fn find_transaction(&self, id: &str) -> Result<Transaction> {
        for block in self.iter() {
            for tx in block.get_transaction() {
                if tx.id == id {
                    return Ok(tx.clone());
                }
            }
        }
        Err(format_err!("Transaction is not found"))
    }

    pub fn find_unspent_transactions(&self, pub_key_hash: &[u8]) -> Vec<Transaction> {
        let mut spent_TXOs: HashMap<String, Vec<i32>> = HashMap::new();
        let mut unspent_TXs: Vec<Transaction> = Vec::new();

        for block in self.iter() {
            for tx in block.get_transaction() {
                for outId in 0..tx.vout.len() {
                    if let Some(ids) = spent_TXOs.get(&tx.id) {
                        if ids.contains(&(outId as i32)) {
                            continue;
                        }
                    }

                    if tx.vout[outId].is_locked_with_key(pub_key_hash) {
                        unspent_TXs.push(tx.to_owned())
                    }
                }

                if !tx.is_coinbase() {
                    for input in &tx.vin {
                        if input.use_key(pub_key_hash) {
                            match spent_TXOs.get_mut(&input.txid) {
                                Some(v) => {
                                    v.push(input.vout);
                                }
                                None => {
                                    spent_TXOs.insert(input.txid.clone(), vec![input.vout]);
                                }
                            }
                        }
                    }
                }
            }
        }

        unspent_TXs
    }

    pub fn find_UTXO(&self, pub_key_hash: &[u8]) -> Vec<TXOutput> {
        let mut unspent_output: Vec<TXOutput> = Vec::new();
        let unspent_transaction = self.find_unspent_transactions(pub_key_hash);

        for tx in unspent_transaction {
            for vout in &tx.vout {
                if vout.is_locked_with_key(pub_key_hash) {
                    unspent_output.push(vout.clone());
                }
            }
        }

        unspent_output
    }

    pub fn find_spendable_outputs(
        &self,
        pub_key_hash: &[u8],
        amount: i32,
    ) -> (i32, HashMap<String, Vec<i32>>) {
        let mut unspent_outputs: HashMap<String, Vec<i32>> = HashMap::new();
        let mut accumulated = 0;
        let unspent_TXs = self.find_unspent_transactions(pub_key_hash);

        for tx in unspent_TXs {
            for index in 0..tx.vout.len() {
                if tx.vout[index].is_locked_with_key(pub_key_hash) && accumulated < amount {
                    match unspent_outputs.get_mut(&tx.id) {
                        Some(v) => v.push(index as i32),
                        None => {
                            unspent_outputs.insert(tx.id.clone(), vec![index as i32]);
                        }
                    }
                    accumulated += tx.vout[index].value;

                    if accumulated >= amount {
                        return (accumulated, unspent_outputs);
                    }
                }
            }
        }
        (accumulated, unspent_outputs)
    }
}

impl<'a> Iterator for BlockchainIterator<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(encoded_block) = self.bc.db.get(&self.current_hash) {
            return match encoded_block {
                Some(b) => {
                    if let Ok(block) = deserialize::<Block>(&b) {
                        self.current_hash = block.get_prev_hash();
                        Some(block)
                    } else {
                        None
                    }
                }
                None => None,
            };
        }
        None
    }
}
