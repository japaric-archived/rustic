[![Build Status](https://travis-ci.org/japaric/rustic.svg?branch=master)](https://travis-ci.org/japaric/rustic)

# rustic

A `rustc` wrapper (written in rust!) than implements the `--run` flag. (This is
my take on
[rust-lang/rust#9826](https://github.com/rust-lang/rust/issues/9826))

## 30-second introduction

```
$ make && make test

# Use the `--run` flag to compile+execute a rust source file
target/rustic -O --run examples/hello.rs
Hello world!

# Note the shebang!
cat examples/hello.rs
#!target/rustic --run

fn main() {
    println!("Hello world!");
}

# Execute a rust file!
examples/hello.rs
Hello world!

# Arguments before `--run` are passed to `rustc`
# (the crate file path is always passed to `rustc`, regardless of its position)
target/rustic --test --run examples/fib.rs

running 2 tests
test test::fib ... ok
test test::fib_10 ... ignored

test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured

# Arguments after `--run` are passed to the produced executable
# (the crate file path is never passed to the executable)
target/rustic examples/fib.rs -O --test --run --bench

running 2 tests
test test::fib ... ignored
test test::fib_10 ... bench:       435 ns/iter (+/- 6)

test result: ok. 0 passed; 0 failed; 1 ignored; 1 measured

# How does it work you ask? See for yourself!
RUST_LOG=rustic=info target/rustic -O --test --run --bench examples/fib.rs
INFO:rustic::tmpdir: `mkdir /tmp/rust-wVElIw`
INFO:rustic: cwd: /tmp/rust-wVElIw | cmd: `rustc '-O' '--test' '/home/japaric/Projects/rustic/examples/fib.rs'`
INFO:rustic: cwd: . | cmd: `/tmp/rust-wVElIw/fib '--bench'`

running 2 tests
test test::fib ... ignored
test test::fib_10 ... bench:       435 ns/iter (+/- 14)

test result: ok. 0 passed; 0 failed; 1 ignored; 1 measured

INFO:rustic::tmpdir: `rm -rf /tmp/rust-wVElIw`

# If the `--run` flag is absent, `rustic` behaves just like `rustc`
target/rustic examples/hello.rs && ./hello && rm hello
Hello world!
```

## Disclaimer

Use at your own risk! And please file an issue if you find any bug.

## License

rustic is dual licensed under the Apache 2.0 license and the MIT license.

See LICENSE-APACHE and LICENSE-MIT for more details.
