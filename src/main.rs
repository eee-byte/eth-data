#![recursion_limit = "128"]

#[macro_use]
extern crate log;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};

mod cli;
mod decode;
mod error;
mod eth;
mod types;

use eth::{EtherScanApi, check_tx};
use types::{H160, H256};

#[macro_use] extern crate hex_literal;

fn main() {
    println!("Hello, world!");
    env_logger::Builder::new()
        .format(|buf, record| {
            let level = buf.default_styled_level(record.level());
            writeln!(
                buf,
                "[{}] {} - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                level,
                record.args()
            )
        })
        .filter_level(log::LevelFilter::Info)
        .init();

    let fp_tx = File::open("tx-hash.txt").unwrap();
    let f_tx = BufReader::new(fp_tx);


    for line_hash in f_tx.lines() {
        let fp_addr = File::open("eth-addr.txt").unwrap();
        let f_eth = BufReader::new(fp_addr);
        if let Ok(hash) = line_hash {
            println!("hash: {:?}", hash);
            let mut hex = hex::decode(hash).unwrap();
            let result = EtherScanApi::new().get_tx_by_hash(H256::from_slice(&hex));

            let (who, from) = check_tx(result.unwrap()).unwrap();
            println!("who: {:?} from: {:?}", who, from);
            for line in f_eth.lines() {
                if let Ok(addr) = line {
                    let addr = H160::from_slice(&hex::decode(addr).unwrap());
                    if from == addr {
                        println!("{:?}", "=================================");
                        println!("from:{:?}", from);
                        println!("addr:{:?}", addr);
                    }
                }
            }
        }
    }
}
