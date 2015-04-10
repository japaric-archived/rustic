[![Build Status][travis]](https://travis-ci.org/japaric/rustic)

# `rustic`

Rustic scripts (`#!/usr/bin/rustic`) with access to the Cargo ecosystem

## One-minute Introduction

`rustic` is a `cargo` wrapper that lets you run rust source files!

```
$ cat hello.rs
fn main() {
    println!("Hello, world!");
}

$ rustic hello.rs
Hello, world!
```

Shebangs also work!

```
$ cat script.rs
#!/usr/bin/rustic

fn main() {
    println!("This is a rust script!");
}

$ ./script.rs
This is a rust script!
```

All the arguments minus the first one will be passed to the resulting binary.

```
$ cat args.rs
#!/usr/bin/rustic

use std::env;

fn main() {
    println!("{:?}", env::args_os().skip(1).collect::<Vec<_>>());
}

$ rustic args.rs a b c
["a", "b", "c"]

$ ./args.rs a b c
["a", "b", "c"]
```

Because `rustic` is `cargo` powered you can access the whole cargo ecosystem by
embedding a `Cargo.toml` in the comments of your source file.

```
$ cat rand.rs
// Cargo.toml
//
// [dependencies]
// rand = "*"

extern crate rand;

use rand::Rng;

fn main() {
    println!("{}", rand::thread_rng().next_u32());
}

$ time rustic rand.rs
rustic rand.rs  5.79s user 0.22s system 89% cpu 6.729 total
3649997390
```

Each `rustic` script is backed by a `cargo` project, so the first run is always
slow, but subsequent executions have minimal start up time.

```
$ time rustic rand.rs
rustic rand.rs  0.00s user 0.00s system 73% cpu 0.005 total
3089271122
```

## Installation and dependencies

`rustic` depends on `cargo`, which depends on `rustc`, so you need to have both
installed. You can install both using [rustup.sh] or [multirust].

[rustup.sh]: https://github.com/rust-lang/rust/blob/master/src/etc/rustup.sh
[multirust]: https://github.com/brson/multirust

There is no `cargo install` (yet), so for now you'll have to manually build the
project and install it.

```
$ git clone --depth 1 https://github.com/japaric/rustic
$ cd rustic
$ cargo build --release
$ sudo cp target/release/rustic /usr/bin
```

## How it works?

- For each source file, `rustic` keeps a *binary* `cargo` project in your cache
  directory (`$HOME/.cache/rustic`). `rustic` derives the name of the cargo
  project from the name of the source file (e.g. `rand.rs` -> `rand`) and uses
  the source file as the project's `main.rs` file. Note that two source files
  with the same name but located in different parts of your filesystem will be
  mapped to the same `cargo` project.

- The project's `Cargo.toml` will be the original one created by `cargo new`
  plus the one found in the comments of the source file. `rustic` expects the
  source's `Cargo.toml` to be in a *single* comment block that starts with
  `// Cargo.toml`, the location of the comment block is not important.

- `rustic` rebuilds the `cargo` project only when the source file has been
  "modified". On each build, `rustic` updates a `time.stamp` file that contains
  the last modification date of the source file. A project has been "modified"
  when its modification date is greater that the one in the `time.stamp`.

You can get a better grasp of the inner workings by looking at the debug logs.

```
$ export RUST_LOG=rustic
$ rustic rand.rs
DEBUG:rustic::cargo: Project::new("rand.rs")
DEBUG:rustic::cargo: Project::new: using absolute path: "/home/japaric/Projects/rustic/examples/rand.rs"
DEBUG:rustic::cargo: Project::new: using "rand" as project name
DEBUG:rustic::cargo: Project::new: project "rand" already exists
DEBUG:rustic::cargo: Project::new: removing old `src/main.rs` file
DEBUG:rustic::cargo: Project::new: OK
DEBUG:rustic::cargo: Project::new: updating `src/main.rs` symlink
DEBUG:rustic::cargo: Project::new: OK
DEBUG:rustic::cargo: Project::modified: opening `src/main.rs`
DEBUG:rustic::cargo: Project::modified: OK
DEBUG:rustic::cargo: Project::modified: `src/main.rs` was last modified on 1428483392438
DEBUG:rustic::cargo: Project::timestamp: reading `time.stamp`
DEBUG:rustic::cargo: Project::timestamp: OK
DEBUG:rustic::cargo: Project::timestamp: got 1428483392438
DEBUG:rustic::cargo: Project::modified: NO, last modified on 1428483392438, timestamp was 1428483392438
DEBUG:rustic::cargo: Project::run: `"/home/japaric/.cache/rustic/rand/target/release/rand"`
2495710899
```

## Problems? Ideas?

Found a bug? Got a cool feature in mind? Documentation is lacking? Please open
an issue to let me know.

## License

rustic is dual licensed under the Apache 2.0 license and the MIT license.

See LICENSE-APACHE and LICENSE-MIT for more details.

[travis]: https://travis-ci.org/japaric/rustic.svg?branch=master
