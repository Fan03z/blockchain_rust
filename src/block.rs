#![allow(unused)]

use super::*;

use bincode::serialize;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sha256;

const TARGET_HEXS: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    timestamp: u128,
    data: String,
    prev_block_hash: String,
    hash: String,
    nonce: i32,
}

impl Block {
    pub fn new_block(data: String, prev_block_hash: String) -> Result<Block> {
        let timestamp = Utc::now().timestamp_millis() as u128;
        let mut block = Block {
            timestamp,
            data,
            prev_block_hash,
            hash: String::new(),
            nonce: 0,
        };
        block.proof_of_work()?;
        Ok(block)
    }

    pub fn new_genesis_block() -> Block {
        Block::new_block(String::from("Genesis Block"), String::new()).unwrap()
    }

    pub fn proof_of_work(&mut self) -> Result<()> {
        info!("Mining the block containing \"{}\"\n", self.data);

        while !self.validate()? {
            self.nonce += 1;
        }

        let data = self.prepare_hash_data()?;
        let hash = sha256::digest(data);
        self.hash = hash;

        Ok(())
    }

    pub fn prepare_hash_data(&self) -> Result<Vec<u8>> {
        let content = (
            self.prev_block_hash.clone(),
            self.data.clone(),
            self.timestamp,
            TARGET_HEXS,
            self.nonce,
        );
        let data = serialize(&content)?;
        Ok(data)
    }

    pub fn validate(&self) -> Result<bool> {
        let data = self.prepare_hash_data()?;
        let mut vec: Vec<u8> = Vec::new();
        vec.resize(TARGET_HEXS, '0' as u8);

        Ok(&sha256::digest(data)[..TARGET_HEXS] == String::from_utf8(vec)?)
    }

    // --------- getter ---------

    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    pub fn get_prev_hash(&self) -> String {
        self.prev_block_hash.clone()
    }
}
