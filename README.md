# rustic

A `rustc` wrapper (written in rust!) than implements the `--run` flag. (This is
my take on
[rust-lang/rust#9826](https://github.com/rust-lang/rust/issues/9826))

## 30-second introduction

```
# Create the `bin/rustic` binary
$ make

# Use the `--run` flag to compile+execute a rust source file
$ bin/rustic -O --run examples/hello.rs
Hello world!

# Note the shebang!
$ cat examples/hello.rs
#!bin/rustic --run

fn main() {
    println!("Hello world!");
}

# Execute a rust file!
$ examples/hello.rs
Hello world!

# Use `--run` in conjunction with `--test` to run a test suite
$ bin/rustic --run --test examples/fib.rs

running 2 tests
test test::fib ... ok
test test::fib_10 ... ignored

test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured

# Guess what `--run` + `--bench` does!
$ bin/rustic -O --bench --run examples/fib.rs

running 2 tests
test test::fib ... ignored
test test::fib_10 ... bench:       437 ns/iter (+/- 8)

test result: ok. 0 passed; 0 failed; 1 ignored; 1 measured

# How does it work you ask? See for yourself!
$ RUST_LOG=rustic=info bin/rustic -O --run --bench examples/fib.rs
INFO:rustic::tmpdir: `mkdir /tmp/rust-4TvaFz`
INFO:rustic: cwd: /tmp/rust-4TvaFz | cmd: `rustc '-O' '--test' '/home/japaric/Projects/rustic/examples/fib.rs'`
INFO:rustic: cwd: . | cmd: `/tmp/rust-4TvaFz/fib '--bench'`

running 2 tests
test test::fib ... ignored
test test::fib_10 ... bench:       435 ns/iter (+/- 15)

test result: ok. 0 passed; 0 failed; 1 ignored; 1 measured

INFO:rustic::tmpdir: `rm -rf /tmp/rust-4TvaFz`

# If the `--run` flag is absent, `rustic` behaves like `rustc`
$ bin/rustic examples/hello.rs && ./hello && rm hello
Hello world!
```

## Disclaimer

Use at your own risk! I just hacked this up, so expect to find bugs. Please
file an issue if you do.

## License

rustic is dual licensed under the Apache 2.0 license and the MIT license.

See LICENSE-APACHE and LICENSE-MIT for more details.
