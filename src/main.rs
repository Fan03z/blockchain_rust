#![allow(unused)]

mod block;
mod blockchain;
mod cli;
mod server;
mod transaction;
mod utxoset;
mod wallets;

extern crate env_logger;

use anyhow::Result;
use log::{debug, error, info};

use crate::blockchain::Blockchain;

fn main() -> Result<()> {
    env_logger::init();

    let mut cli = cli::Cli::new();
    if let Err(e) = cli.run() {
        println!("Error: {}", e);
    };

    Ok(())
}
