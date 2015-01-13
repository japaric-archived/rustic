RUSTIC = target/release/rustic
PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin

.PHONY: all test

all:
	cargo build --release

test:
	# Use the `--run` flag to compile+execute a rust source file
	$(RUSTIC) -O example/hello.rs --run
	# Note the shebang!
	cat example/hello.rs
	# Execute a rust file!
	example/hello.rs
	# Run your test suite
	$(RUSTIC) example/fib.rs --run --test
	# Arguments after the `--` are passed to the produced executable
	$(RUSTIC) -O example/fib.rs --run --test -- --bench
	# How does it work you ask? See for yourself!
	RUST_LOG=rustic=info $(RUSTIC) -O example/fib.rs --run --test -- --bench
	# If the `--run` flag is absent, `rustic` behaves just like `rustc`
	$(RUSTIC) example/hello.rs && ./hello && rm hello
