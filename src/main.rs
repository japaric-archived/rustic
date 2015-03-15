#![deny(warnings)]
#![feature(exit_status)]
#![feature(path_ext)]

extern crate env_logger;
#[macro_use]
extern crate log;
extern crate tempdir;

use std::env;
use std::fs::{PathExt, self};
use std::path::Path;
use std::process::{Command, Stdio};

use tempdir::TempDir;

fn main() {
    env_logger::init().unwrap();

    let args: Vec<_> = env::args().skip(1).collect();

    // If `--run` is not in the arguments: pass all the arguments to `rustc`
    if args.iter().all(|arg| *arg != "--run") {
        let mut cmd = Command::new("rustc");
        cmd.args(&args);

        // Inherit stdio
        cmd.stderr(Stdio::inherit());
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());

        info!("cwd: . | cmd: `{:?}`", cmd);
        match cmd.output() {
            Err(e) => panic!("`{:?}` failed: {}", cmd, e),
            Ok(output) =>  {
                env::set_exit_status(output.status.code().unwrap());

                return;
            },
        }
    }

    // Before `--`: arguments for the compiler
    // After `--`: arguments for the executable
    let mut splitted_args = args.split(|arg| *arg == "--");
    let compiler_args = splitted_args.next().unwrap();
    let executable_args = splitted_args.next();

    // TODO `--crate-type=lib` should also be forbidden
    // XXX What if the crate has a `crate_type=*lib` attribute?
    if compiler_args.iter().any(|arg| *arg == "--out-dir") {
        panic!("Can't use both `--out-dir` and `--run` flags at the same time");
    }

    // Build the rustc command
    let mut cmd = Command::new("rustc");
    // Make all paths absolute, filter out the `--run` flag
    let current_dir = env::current_dir().ok().expect("Couldn't fetch the current directory");
    for arg in compiler_args.iter().filter(|arg| **arg != "--run") {
        let path = Path::new(arg);

        if path.exists() && path.is_relative() {
            cmd.arg(&current_dir.join(path));
        } else {
            cmd.arg(arg);
        }
    }

    // Create temporary directory
    let temp_dir = TempDir::new("rust").unwrap();
    let temp_dir_path = temp_dir.path();

    // Inherit stdio
    cmd.stderr(Stdio::inherit());
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());

    // Compile
    info!("cwd: {:?} | cmd: `{:?}`", temp_dir_path, cmd);
    match cmd.current_dir(temp_dir_path).output() {
        Err(e) => panic!("`{:?}` failed: {}", cmd, e),
        Ok(output) => if !output.status.success() {
            env::set_exit_status(output.status.code().unwrap());

            return;
        },
    }

    // Look for the produced binary
    let mut cmd = match fs::read_dir(temp_dir_path) {
        Err(e) => panic!("`ls {:?}` failed: {}", temp_dir_path, e),
        Ok(mut paths) => match paths.next() {
            Some(Ok(entry)) => Command::new(&entry.path()),
            _ => panic!("no binary found in {:?}", temp_dir_path),
        }
    };

    // Build the executable command
    match executable_args {
        None => {},
        Some(args) => {
            cmd.args(args);
        },
    }

    // Inherit stdio
    cmd.stderr(Stdio::inherit());
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());

    // Execute
    info!("cwd: . | cmd: `{:?}`", cmd);
    match cmd.output() {
        Err(e) => panic!("`{:?}` failed: {}", cmd, e),
        Ok(output) => if !output.status.success() {

            env::set_exit_status(output.status.code().unwrap());

            return;
        },
    }
}
