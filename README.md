[![Build Status][travis]](https://travis-ci.org/japaric/rustic)

# rustic

A `rustc` wrapper (written in rust!) than implements the `--run` flag. (This is
my take on [rust-lang/rust#9826][issue])

## 30-second introduction

```
$ make && make test

# Use the `--run` flag to compile+execute a rust source file
target/release/rustic -O examples/hello.rs --run
Hello world!
# Note the shebang!
cat examples/hello.rs
#!target/release/rustic --run

fn main() {
    println!("Hello world!");
}
# Execute a rust file!
examples/hello.rs
Hello world!
# Run your test suite
target/release/rustic examples/fib.rs --run --test

running 2 tests
test test::fib ... ok
test test::fib_10 ... ignored

test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured

# Arguments after the `--` are passed to the produced executable
target/release/rustic -O examples/fib.rs --run --test -- --bench

running 2 tests
test test::fib ... ignored
test test::fib_10 ... bench:       463 ns/iter (+/- 10)

test result: ok. 0 passed; 0 failed; 1 ignored; 1 measured

# How does it work you ask? See for yourself!
RUST_LOG=rustic=info target/release/rustic -O examples/fib.rs --run --test -- --bench
INFO:rustic: cwd: /tmp/rs-19128-0-rust | cmd: `rustc '-O' '/home/japaric/Projects/rustic/examples/fib.rs' '--test'`
INFO:rustic: cwd: . | cmd: `/tmp/rs-19128-0-rust/fib '--bench'`

running 2 tests
test test::fib ... ignored
test test::fib_10 ... bench:       464 ns/iter (+/- 12)

test result: ok. 0 passed; 0 failed; 1 ignored; 1 measured

# If the `--run` flag is absent, `rustic` behaves just like `rustc`
target/release/rustic examples/hello.rs && ./hello && rm hello
Hello world!
```

## License

rustic is dual licensed under the Apache 2.0 license and the MIT license.

See LICENSE-APACHE and LICENSE-MIT for more details.

[issue]: https://github.com/rust-lang/rust/issues/9826
[travis]: https://travis-ci.org/japaric/rustic.svg?branch=master
