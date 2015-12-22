//! Rustic scripts (`#!/usr/bin/rustic`) with access to the Cargo ecosystem

#![deny(missing_docs)]
#![deny(warnings)]

#[macro_use] extern crate log;

extern crate env_logger;
extern crate lines;

use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;
use std::{fmt, io};

use cargo::Project;

pub mod cargo;

/// Errors
#[derive(Debug)]
pub enum Error {
    /// `cargo build` failed
    CargoBuild(Vec<u8>),
    /// `cargo new` failed
    CargoNew(Vec<u8>),
    /// IO error
    Io(io::Error),
    /// Logger initialization failed
    Log(log::SetLoggerError),
    /// `time.stamp` is malformed
    MalformedTimestamp(String),
    /// No arguments passed
    NoArgs,
    /// Can't find cache directory
    NoCacheDir,
    /// First argument is not a file
    NotAFile(PathBuf),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;

        match *self {
            CargoBuild(ref stderr) => {
                let stderr = String::from_utf8_lossy(stderr);
                write!(f, "`cargo build` failed with:\n{}`", stderr)
            },
            CargoNew(ref stderr) => {
                let stderr = String::from_utf8_lossy(stderr);
                write!(f, "`cargo new` failed with:\n{}", stderr)
            },
            Io(ref e) => e.fmt(f),
            Log(ref e) => e.fmt(f),
            MalformedTimestamp(ref s) => write!(f, "malformed timestamp: {}", s),
            NoArgs => f.write_str("Expected path to source file as first argument"),
            NoCacheDir => f.write_str("Couldn't find cache directory (is $HOME not set?)"),
            NotAFile(ref path) => write!(f, "{:?} is not a file", path),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<log::SetLoggerError> for Error {
    fn from(e: log::SetLoggerError) -> Error {
        Error::Log(e)
    }
}

fn help() {
    println! {"\
Usage: rustic FILE [ARG]...
Usage: rustic OPTION

Options:
  -h, --help        display this help and exit
  -V, --version     output version information and exit
"
    }
}

fn init_logger() -> Result<(), Error> {
    try!(env_logger::init());
    Ok(())
}

/// Entry point for the `rustic` binary
pub fn run() -> Result<Option<i32>, Error> {
    try!(init_logger());

    let mut args = env::args_os().skip(1);

    let arg = try!(args.next().ok_or(Error::NoArgs));

    if arg.as_os_str() == OsStr::new("--version") || arg.as_os_str() == OsStr::new("-V") {
        try!(try!(Command::new("rustc").arg("-V").spawn()).wait());
        try!(try!(Command::new("cargo").arg("-V").spawn()).wait());

        return Ok(None)
    } else if arg.as_os_str() == OsStr::new("-h") || arg.as_os_str() == OsStr::new("--help") {
        help();

        return Ok(None)
    }

    let project = try!(Project::new(PathBuf::from(arg)));

    if try!(project.has_changed()) {
        try!(project.update_cargo_toml());
        try!(project.remove_lock());
        try!(project.build());
        try!(project.update_timestamp());
    }

    let output = try!(project.run(args));

    Ok(output.status.code())
}
