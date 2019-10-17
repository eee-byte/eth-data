#![recursion_limit = "128"]

#[macro_use]
extern crate log;

mod error;
mod decode;
mod cli;
mod eth;
mod types;

fn main() {
    println!("Hello, world!");
}
