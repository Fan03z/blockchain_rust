#![allow(unused)]

use clap::{arg, command, Arg, Command};

use super::*;

pub struct Cli {
    bc: Blockchain,
}

impl Cli {
    pub fn new() -> Result<Cli> {
        Ok(Cli {
            bc: Blockchain::new()?,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        info!("run blockchain_rust node");

        let matches = command!("blockchain_rust")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(Command::new("printchain").about("print all the chain blocks"))
            .subcommand(
                Command::new("addblock")
                    .about("add a block in the blockchain")
                    .arg(arg!([data])),
            )
            .get_matches();

        match matches.subcommand() {
            Some(("printchain", sub_matches)) => {
                self.print_chain();
            }
            Some(("addblock", sub_matches)) => {
                let data = sub_matches
                    .get_one::<String>("data")
                    .expect("Need data argument");
                self.addblock(String::from(&data[..]));
            }
            _ => {
                panic!("Input command is wrong");
            }
        }

        Ok(())
    }

    fn print_chain(&mut self) {
        for b in &mut self.bc {
            println!("{:#?}", b);
        }
    }

    fn addblock(&mut self, data: String) -> Result<()> {
        self.bc.add_block(data)
    }
}
