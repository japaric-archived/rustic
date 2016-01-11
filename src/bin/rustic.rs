extern crate rustic;

use std::process::exit;

fn main() {
    match rustic::run() {
        Err(e) => {
            println!("{}", e);
            exit(1);
        }
        Ok(Some(status)) => {
            exit(status);
        }
        Ok(None) => {
            exit(0);
        }
    }
}
