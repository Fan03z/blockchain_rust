#![allow(unused)]

use bincode::{deserialize, serialize};
use sled;

use super::*;
use crate::block::Block;

#[derive(Debug, Clone)]
pub struct Blockchain {
    tip: String,
    current_hash: String,
    db: sled::Db,
}

impl Blockchain {
    pub fn new() -> Result<Blockchain> {
        info!("Create new blockchain");

        let mut db = sled::open("./db")?;

        match db.get("LAST")? {
            Some(hash) => {
                info!("Found block database");
                let last_hash = String::from_utf8(hash.to_vec())?;
                Ok(Blockchain {
                    tip: last_hash.clone(),
                    current_hash: last_hash,
                    db: db,
                })
            }
            None => {
                info!("Creating new block database");
                let block = Block::new_genesis_block();
                db.insert(block.get_hash(), serialize(&block)?)?;
                db.insert("LAST", block.get_hash().as_bytes())?;
                let blockchain = Blockchain {
                    tip: block.get_hash(),
                    current_hash: block.get_hash(),
                    db: db,
                };
                blockchain.db.flush()?;

                Ok(blockchain)
            }
        }
    }

    pub fn add_block(&mut self, data: String) -> Result<()> {
        info!("Add new block to the chain");
        let last_hash = self.db.get("LAST")?.unwrap();
        let newblock = Block::new_block(data, String::from_utf8(last_hash.to_vec())?)?;

        self.db.insert(newblock.get_hash(), serialize(&newblock)?)?;
        self.db.insert("LAST", newblock.get_hash().as_bytes())?;
        self.db.flush()?;

        self.tip = newblock.get_hash();
        self.current_hash = newblock.get_hash();

        Ok(())
    }
}

impl Iterator for Blockchain {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(encoded_block) = self.db.get(&self.current_hash) {
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
