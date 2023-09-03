#![allow(unused)]

use super::*;

use anyhow::Result;
use bincode::serialize;
use chrono::prelude::*;
use sha256;

#[derive(Debug)]
pub struct Block {
    timestamp: u128,
    data: String,
    prev_block_hash: String,
    hash: String,
}

impl Block {
    pub fn new_block(data: String, prev_block_hash: String) -> Result<Block> {
        let mut block = Block {
            timestamp: 0,
            data,
            prev_block_hash,
            hash: String::new(),
        };
        block.set_hash()?;
        Ok(block)
    }

    pub fn new_genesis_block() -> Block {
        Block::new_block(String::from("Genesis Block"), String::new()).unwrap()
    }

    pub fn set_hash(&mut self) -> Result<()> {
        self.timestamp = Utc::now().timestamp_millis() as u128;
        let content = (self.data.clone(), self.timestamp);
        let bytes = serialize(&content)?;
        let hash = sha256::digest(&bytes[..]);
        self.hash = hash;
        Ok(())
    }

    // --------- getter ---------

    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }
}
