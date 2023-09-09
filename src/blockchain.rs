#![allow(unused)]
#![allow(non_snake_case)]

use anyhow::format_err;
use bincode::{deserialize, serialize};
use sled;

use super::*;
use crate::block::Block;
use crate::transaction::{TXOutput, TXOutputs, Transaction};
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
        let last_hash = if hash.is_empty() {
            String::new()
        } else {
            String::from_utf8(hash.to_vec())?
        };
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

    pub fn mine_block(&mut self, transactions: Vec<Transaction>) -> Result<Block> {
        info!("mine a new block");

        for tx in &transactions {
            if !self.verify_transaction(&tx)? {
                return Err(format_err!("ERROR: Invalid transaction"));
            }
        }

        let last_hash = self.db.get("LAST")?.expect("Could not get last hash ");
        let new_block = Block::new_block(transactions, String::from_utf8(last_hash.to_vec())?)?;
        self.db
            .insert(new_block.get_hash(), serialize(&new_block)?)?;
        self.db.insert("LAST", new_block.get_hash().as_bytes())?;
        self.db.flush()?;

        self.tip = new_block.get_hash();

        Ok(new_block)
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

    pub fn find_UTXO(&self) -> HashMap<String, TXOutputs> {
        let mut utxos: HashMap<String, TXOutputs> = HashMap::new();
        let mut spend_txos: HashMap<String, Vec<i32>> = HashMap::new();

        for block in self.iter() {
            for tx in block.get_transaction() {
                for index in 0..tx.vout.len() {
                    if let Some(ids) = spend_txos.get(&tx.id) {
                        if ids.contains(&(index as i32)) {
                            continue;
                        }
                    }

                    match utxos.get_mut(&tx.id) {
                        Some(v) => v.outputs.push(tx.vout[index].clone()),
                        None => {
                            utxos.insert(
                                tx.id.clone(),
                                TXOutputs {
                                    outputs: vec![tx.vout[index].clone()],
                                },
                            );
                        }
                    }
                }

                if !tx.is_coinbase() {
                    for i in &tx.vin {
                        match spend_txos.get_mut(&i.txid) {
                            Some(v) => {
                                v.push(i.vout);
                            }
                            None => {
                                spend_txos.insert(i.txid.clone(), vec![i.vout]);
                            }
                        }
                    }
                }
            }
        }

        utxos
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
