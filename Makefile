RUSTC = rustc -O src/main.rs
PREFIX = /usr/local

.PHONY: all clean install

all:
	mkdir -p bin
	$(RUSTC) --out-dir bin

clean:
	rm -rf bin

install:
	install -m 0755 bin/rustic $(DESTDIR)/$(PREFIX)/bin

test:
	bin/rustic -O --run examples/hello.rs
	cat examples/hello.rs
	examples/hello.rs
	bin/rustic --run --test examples/fib.rs
	bin/rustic -O --bench --run examples/fib.rs
	RUST_LOG=rustic=info bin/rustic -O --run --bench examples/fib.rs
