//! Rustic scripts (`#!/usr/bin/rustic`) with access to the Cargo ecosystem

#![deny(missing_docs)]
#![deny(warnings)]

#[macro_use] extern crate log;
#[macro_use] extern crate clap;
#[macro_use] extern crate sha1;

extern crate env_logger;

use std::path::PathBuf;
use std::process::Command;
use std::{fmt, io};

use cargo::Project;
use clap::{Arg, App, ClapErrorType};

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
    /// Wrong arguments passed
    WrongArgs(clap::ClapError),
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
            WrongArgs(ref e) => e.fmt(f),
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

impl From<clap::ClapError> for Error {
    fn from(e: clap::ClapError) -> Error {
        Error::WrongArgs(e)
    }
}

fn init_logger() -> Result<(), Error> {
    try!(env_logger::init());
    Ok(())
}

/// Entry point for the `rustic` binary
pub fn run() -> Result<Option<i32>, Error> {
    try!(init_logger());

    let clap_result = App::new("rustic")
        .version(&crate_version!()[..])
        .author("Jorge Aparicio <japaricious@gmail.com>")
        .about("Cargo wrapper that lets you run Rust source files like scripts")
        .arg(Arg::with_name("FILE")
            .index(1)
            .help("Rust source file to compile and run")
            .required(true)
        )
        .arg(Arg::with_name("ARGS")
            .index(2)
            .multiple(true)
            .help("Arguments for the compiled program")
            .required(false)
        )
        .get_matches_safe();

    match clap_result {
        Err(err) => match err.error_type {
            ClapErrorType::VersionDisplayed => {
                try!(try!(Command::new("rustc").arg("-V").spawn()).wait());
                try!(try!(Command::new("cargo").arg("-V").spawn()).wait());
                Ok(None)
            },
            ClapErrorType::HelpDisplayed => Ok(None),
            _ => Err(err.into())
        },
        Ok(matches) => {
            let project = try!(Project::new(PathBuf::from(matches.value_of("FILE").unwrap())));

            if try!(project.has_changed()) {
                try!(project.update_cargo_toml());
                try!(project.remove_lock());
                try!(project.build());
                try!(project.update_timestamp());
            }

            let output = try!(project.run(matches.values_of("ARGS").unwrap_or(vec![])));

            Ok(output.status.code())
        }
    }
}
