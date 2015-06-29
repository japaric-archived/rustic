#![feature(exit_status)]

extern crate rustic;

use std::env;
use std::process::exit;

fn main() {
    match rustic::run() {
        Err(e) => {
            println!("{}", e);
            exit(1);
        },
        Ok(Some(code)) => {
            exit(code)
        },
        _ => {},
    }
}
