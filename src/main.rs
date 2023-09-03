#![allow(unused)]

mod block;
mod blockchain;

use anyhow::Result;

use crate::blockchain::Blockchain;

fn main() -> Result<()> {
    let mut blockchain = Blockchain::new();

    blockchain.add_block(String::from("Send 1 BTC to Ivan"))?;
    blockchain.add_block(String::from("Send 2 more BTC to Ivan"))?;

    println!("Blockchain: {:#?}", blockchain);

    Ok(())
}
