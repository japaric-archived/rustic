use std::env;

fn main() {
    println!("{:?}", env::args_os().skip(1).collect::<Vec<_>>());
}
