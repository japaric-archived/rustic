#![feature(exit_status)]

extern crate rustic;

use std::env;

fn main() {
    match rustic::run() {
        Err(e) => {
            println!("{}", e);
            env::set_exit_status(1);
        },
        Ok(Some(code)) => {
            env::set_exit_status(code)
        },
        _ => {},
    }
}
