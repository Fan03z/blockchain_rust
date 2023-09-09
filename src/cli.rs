#![allow(unused)]

use anyhow::format_err;
use bitcoincash_addr::Address;
use clap::{arg, command, Arg, Command};

use super::*;
use crate::transaction::Transaction;
use crate::utxoset::UTXOSet;
use crate::wallets::Wallets;
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
                Command::new("createwallet")
                    .about("create a wallet with address, private key and public key"),
            )
            .subcommand(Command::new("listaddresses").about("list all address"))
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
                        let bc = Blockchain::create_blockchain(address.clone())?;
                        let utxo_set = UTXOSet { blockchain: bc };
                        utxo_set.reindex()?;
                        println!("Create blockchain");
                    }
                    None => {
                        return Err(format_err!("Need <address> argument"));
                    }
                }
            }
            Some(("getbalance", sub_matches)) => match sub_matches.get_one::<String>("address") {
                Some(address) => {
                    let pub_key_hash = Address::decode(&address).unwrap().body;
                    let bc = Blockchain::new()?;
                    let utxo_set = UTXOSet { blockchain: bc };
                    let utxos = utxo_set.find_UTXO(&pub_key_hash)?;

                    let mut balance = 0;
                    for out in utxos.outputs {
                        balance += out.value;
                    }
                    println!("Balance of '{}': {}\n", address, balance);
                }
                None => {
                    return Err(format_err!("Need <address> argument"));
                }
            },
            Some(("createwallet", sub_matches)) => {
                let mut ws = Wallets::new()?;
                let address = ws.create_wallet();
                ws.save_all()?;
                println!("Create wallet success: address {}", address);
            }
            Some(("listaddresses", sub_matches)) => {
                let ws = Wallets::new()?;
                let addresses = ws.get_all_addresses();
                println!("addresses: ");
                for ad in addresses {
                    println!("{}", ad);
                }
            }
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
                let mut utxo_set = UTXOSet { blockchain: bc };
                let tx = Transaction::new_UTXO(from, to, amount, &utxo_set)?;
                let cbtx = Transaction::new_coinbase(from.to_string(), String::from("reward"))?;

                let new_block = utxo_set.blockchain.mine_block(vec![cbtx, tx])?;
                utxo_set.update(&new_block)?;

                println!("Send transaction success!");
            }
            _ => {
                return Err(format_err!("Invalid Command"));
            }
        }

        Ok(())
    }
}
