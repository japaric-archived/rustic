RUSTC = rustc -O
PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin

.PHONY: all clean install

all:
	mkdir -p bin
	$(RUSTC) --out-dir bin src/main.rs

clean:
	rm -rf bin

install:
	install -d -m 0755 $(DESTDIR)/$(BINDIR)
	install -m 0755 bin/rustic $(DESTDIR)/$(BINDIR)

test:
	bin/rustic -O --run examples/hello.rs
	cat examples/hello.rs
	examples/hello.rs
	bin/rustic --test --run examples/fib.rs
	bin/rustic examples/fib.rs -O --test --run --bench
	RUST_LOG=rustic=info bin/rustic -O --test --run --bench examples/fib.rs
