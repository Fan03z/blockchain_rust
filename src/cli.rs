#![allow(unused)]

use anyhow::format_err;
use bitcoincash_addr::Address;
use clap::{arg, command, ArgAction, Command};

use super::*;
use crate::server::*;
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
            .subcommand(Command::new("reindex").about("reindex unspent-transaction-output set"))
            .subcommand(
                Command::new("startnode")
                    .about("start the node server")
                    .arg(arg!([port]).help("the port server bind to locally")),
            )
            .subcommand(
                Command::new("minernode")
                    .about("start the miner node")
                    .arg(arg!([port]).help("the port server bind to locally"))
                    .arg(arg!([address]).help("the wallet address to accept mining reward")),
            )
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
                    .arg(arg!([amount]).help("Amount to send"))
                    .arg(
                        arg!([mine])
                            .short('m')
                            .long("mine")
                            .action(ArgAction::SetFalse)
                            .help("if mined immediately in the same node when sending transaction"),
                    ),
            )
            .get_matches();

        match matches.subcommand() {
            Some(("printchain", sub_matches)) => {
                let bc = Blockchain::new()?;
                for b in bc.iter() {
                    println!("{:#?}", b);
                }
            }
            Some(("reindex", sub_matches)) => {
                let bc = Blockchain::new()?;
                let utxo_set = UTXOSet { blockchain: bc };
                utxo_set.reindex()?;
                let count = utxo_set.count_transactions()?;
                println!("Done! There are {} transactions in the UTXO set.", count);
            }
            Some(("startnode", sub_matches)) => {
                if let Some(port) = sub_matches.get_one::<String>("port") {
                    println!("Start node...");
                    let bc = Blockchain::new()?;
                    let utxo_set = UTXOSet { blockchain: bc };
                    let server = Server::new(port, "", utxo_set)?;
                    server.start_server()?;
                }
            }
            Some(("minernode", sub_matches)) => {
                let port = if let Some(port) = sub_matches.get_one::<String>("port") {
                    port
                } else {
                    println!("Start miner node need <port> <address> arguments");
                    exit(1);
                };
                let address = if let Some(address) = sub_matches.get_one::<String>("address") {
                    address
                } else {
                    println!("Start miner node need <port> <address> arguments");
                    exit(1);
                };

                println!("Start miner node...");
                let bc = Blockchain::new()?;
                let utxo_set = UTXOSet { blockchain: bc };
                let server = Server::new(port, address, utxo_set)?;
                server.start_server()?;
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

                if sub_matches.get_flag("mine") {
                    let cbtx = Transaction::new_coinbase(from.to_string(), String::from("reward"))?;
                    let new_block = utxo_set.blockchain.mine_block(vec![cbtx, tx])?;
                    utxo_set.update(&new_block)?;
                    println!("Send transaction and wait to add in next block!");
                } else {
                    Server::send_transaction(&tx, utxo_set)?;
                    println!("Send transaction success and add in new mine block!");
                }
            }
            _ => {
                return Err(format_err!("Invalid Command"));
            }
        }

        Ok(())
    }
}
