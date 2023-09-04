#![allow(unused)]

mod block;
mod blockchain;
mod cli;

extern crate env_logger;

use anyhow::Result;
use log::info;

use crate::blockchain::Blockchain;

fn main() -> Result<()> {
    env_logger::init();

    let mut cli = cli::Cli::new()?;
    cli.run()?;

    Ok(())
}
