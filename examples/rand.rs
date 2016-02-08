// Cargo.toml
//
// rand = "*"

extern crate rand;

use rand::Rng;

fn main() {
    println!("{}", rand::thread_rng().next_u32());
}
