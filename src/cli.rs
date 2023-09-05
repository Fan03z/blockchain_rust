#![allow(unused)]

use anyhow::format_err;
use clap::{arg, command, Arg, Command};

use super::*;
use crate::transaction::Transaction;
use std::process::exit;

pub struct Cli {}

impl Cli {
    pub fn new() -> Cli {
        Cli {}
    }

    pub fn run(&mut self) -> Result<()> {
        info!("run blockchain_rust node");

        let matches = command!("blockchain_rust")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(Command::new("printchain").about("print all the chain blocks"))
            .subcommand(
                Command::new("createblockchain")
                    .about("create a new blockchain and ")
                    .arg(arg!([address]).help("which address will get the genesis reward")),
            )
            .subcommand(
                Command::new("getbalance")
                    .about("get the address balance in the blockchain")
                    .arg(arg!([address]).help("Which address's balance")),
            )
            .subcommand(
                Command::new("send")
                    .about("send in blockchain")
                    .arg(arg!([from]).help("Source wallet address"))
                    .arg(arg!([to]).help("Destination wallet address"))
                    .arg(arg!([amount]).help("Amount to send")),
            )
            .get_matches();

        match matches.subcommand() {
            Some(("printchain", sub_matches)) => {
                let bc = Blockchain::new()?;
                for b in bc.iter() {
                    println!("{:#?}", b);
                }
            }
            Some(("createblockchain", sub_matches)) => {
                match sub_matches.get_one::<String>("address") {
                    Some(address) => {
                        let address = String::from(&address[..]);
                        Blockchain::create_blockchain(address.clone())?;
                        println!("Create blockchain");
                    }
                    None => {
                        return Err(format_err!("Need <address> argument"));
                    }
                }
            }
            Some(("getbalance", sub_matches)) => match sub_matches.get_one::<String>("address") {
                Some(address) => {
                    let address = String::from(&address[..]);
                    let bc = Blockchain::new()?;
                    let utxos = bc.find_UTXO(&address);
                    let mut balance = 0;
                    for utxo in utxos {
                        balance += utxo.value;
                    }
                    println!("Balance of '{}': {}\n", address, balance);
                }
                None => {
                    return Err(format_err!("Need <address> argument"));
                }
            },
            Some(("send", sub_matches)) => {
                let from = if let Some(from_address) = sub_matches.get_one::<String>("from") {
                    from_address
                } else {
                    println!("Send transaction need <from> <to> <amount> arguments");
                    exit(1);
                };
                let to = if let Some(to_address) = sub_matches.get_one::<String>("to") {
                    to_address
                } else {
                    println!("Send transaction need <from> <to> <amount> arguments");
                    exit(1);
                };
                let amount: i32 = if let Some(amount) = sub_matches.get_one::<String>("amount") {
                    String::from(&amount[..]).parse()?
                } else {
                    println!("Send transaction need <from> <to> <amount> arguments");
                    exit(1);
                };

                let mut bc = Blockchain::new()?;
                let tx = Transaction::new_UTXO(from, to, amount, &bc)?;
                bc.mine_block(vec![tx])?;
                println!("Send transaction success!");
            }
            _ => {
                return Err(format_err!("Invalid Command"));
            }
        }

        Ok(())
    }
}
