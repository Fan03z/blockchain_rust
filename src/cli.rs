#![allow(unused)]

use anyhow::format_err;
use bitcoincash_addr::Address;
use clap::{arg, command, Arg, Command};

use super::*;
use crate::transaction::Transaction;
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
                    let pub_key_hash = Address::decode(&address).unwrap().body;
                    let bc = Blockchain::new()?;
                    let utxos = bc.find_UTXO(&pub_key_hash);
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
