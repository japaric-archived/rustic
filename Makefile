RUSTC = rustc -O
PREFIX = /usr/local

.PHONY: all clean install

all:
	mkdir -p bin
	$(RUSTC) --out-dir bin src/main.rs

clean:
	rm -rf bin

install:
	install -m 0755 bin/rustic $(DESTDIR)/$(PREFIX)/bin

test:
	bin/rustic -O --run examples/hello.rs
	cat examples/hello.rs
	examples/hello.rs
	bin/rustic --test --run examples/fib.rs
	bin/rustic examples/fib.rs -O --test --run --bench
	RUST_LOG=rustic=info bin/rustic -O --test --run --bench examples/fib.rs
