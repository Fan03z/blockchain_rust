#![allow(unused)]

use super::*;
use crate::transaction::Transaction;

use bincode::serialize;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sha256;

const TARGET_HEXS: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    timestamp: u128,
    transactions: Vec<Transaction>,
    prev_block_hash: String,
    hash: String,
    nonce: i32,
    height: i32,
}

impl Block {
    pub fn new_block(
        transactions: Vec<Transaction>,
        prev_block_hash: String,
        height: i32,
    ) -> Result<Block> {
        let timestamp = Utc::now().timestamp_millis() as u128;
        let mut block = Block {
            timestamp,
            transactions,
            prev_block_hash,
            hash: String::new(),
            nonce: 0,
            height,
        };
        block.proof_of_work()?;
        Ok(block)
    }

    pub fn new_genesis_block(coinbase: Transaction) -> Block {
        Block::new_block(vec![coinbase], String::new(), 0).unwrap()
    }

    pub fn proof_of_work(&mut self) -> Result<()> {
        info!("Mining the block containing \"{:#?}\"\n", self.transactions);

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
            self.transactions.clone(),
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

    pub fn get_transaction(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }
}
