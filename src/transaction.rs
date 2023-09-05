#![allow(non_snake_case)]

use super::*;

use anyhow::format_err;
use bincode::serialize;
use serde::{Deserialize, Serialize};
use sha256;

const SUBSIDY: i32 = 10;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TXInput {
    pub txid: String,
    pub vout: i32,
    pub script_sig: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TXOutput {
    pub value: i32,
    pub script_pub_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    pub id: String,
    pub vin: Vec<TXInput>,
    pub vout: Vec<TXOutput>,
}

impl TXInput {
    pub fn can_unlock_output_with(&self, unlocking_data: &str) -> bool {
        self.script_sig == unlocking_data
    }
}

impl TXOutput {
    pub fn can_be_unlock_with(&self, unlocking_data: &str) -> bool {
        self.script_pub_key == unlocking_data
    }
}

impl Transaction {
    pub fn new_UTXO(from: &str, to: &str, amount: i32, bc: &Blockchain) -> Result<Transaction> {
        info!("new UTXO Transaction from: {} to: {}", from, to);
        let mut vin = Vec::new();
        let acc_uo = bc.find_spendable_outputs(from, amount);

        if acc_uo.0 < amount {
            error!("Not Enough balance");
            return Err(format_err!(
                "Not Enough balance: current balance {}",
                acc_uo.0
            ));
        }

        for tx in acc_uo.1 {
            for out in tx.1 {
                let input = TXInput {
                    txid: tx.0.clone(),
                    vout: out,
                    script_sig: String::from(from),
                };
                vin.push(input);
            }
        }

        let mut vout = vec![TXOutput {
            value: amount,
            script_pub_key: String::from(to),
        }];
        if acc_uo.0 > amount {
            vout.push(TXOutput {
                value: acc_uo.0 - amount,
                script_pub_key: String::from(from),
            })
        }

        let mut tx = Transaction {
            id: String::new(),
            vin,
            vout,
        };
        tx.set_id()?;
        Ok(tx)
    }

    pub fn new_coinbase(to: String, mut data: String) -> Result<Transaction> {
        info!("new coinbase Transaction to: {}", to);
        if data == String::from("") {
            data += &format!("Reward to {}", to);
        }

        let mut tx = Transaction {
            id: String::new(),
            vin: vec![TXInput {
                txid: String::new(),
                vout: -1,
                script_sig: data,
            }],
            vout: vec![TXOutput {
                value: SUBSIDY,
                script_pub_key: to,
            }],
        };
        tx.set_id()?;
        Ok(tx)
    }

    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.is_empty() && self.vin[0].vout == -1
    }

    pub fn set_id(&mut self) -> Result<()> {
        let hash = sha256::digest(serialize(self)?);
        self.id = hash;
        Ok(())
    }
}
