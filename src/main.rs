#![allow(unstable)]
#![deny(warnings)]

#[macro_use]
extern crate log;

use std::io::TempDir;
use std::io::fs::{PathExtensions, self};
use std::io::process::{Command, ExitSignal, ExitStatus, InheritFd, ProcessOutput};
use std::os;

fn main() {
    let args = os::args();
    let args = args.slice_from(1);

    // If `--run` is not in the arguments: pass all the arguments to `rustc`
    if args.iter().all(|arg| arg.as_slice() != "--run") {
        let mut cmd = Command::new("rustc");
        cmd.args(args);
        cmd.stdout(InheritFd(1));
        cmd.stderr(InheritFd(2));

        info!("cwd: . | cmd: `{}`", cmd);
        match cmd.output() {
            Err(e) => panic!("`{}` failed: {}", cmd, e),
            Ok(ProcessOutput { status: exit, .. }) =>  {
                let exit_code = match exit {
                    ExitSignal(code) => code,
                    ExitStatus(code) => code,
                };

                os::set_exit_status(exit_code);

                return;
            },
        }
    }

    // Before `--`: arguments for the compiler
    // After `--`: arguments for the executable
    let mut splitted_args = args.split(|arg| arg.as_slice() == "--");
    let compiler_args = splitted_args.next().unwrap();
    let executable_args = splitted_args.next();

    // TODO `--crate-type=lib` should also be forbidden
    // XXX What if the crate has a `crate_type=*lib` attribute?
    if compiler_args.iter().any(|arg| arg.as_slice() == "--out-dir") {
        panic!("Can't use both `--out-dir` and `--run` flags at the same time");
    }

    // Build the rustc command
    let mut cmd = Command::new("rustc");
    // Make all paths absolute, filter out the `--run` flag
    let current_dir = os::getcwd().ok().expect("Couldn't fetch the current directory");
    for arg in compiler_args.iter().map(|arg| arg.as_slice()).filter(|&arg| arg != "--run") {
        let path = Path::new(arg);
        if path.exists() && path.is_relative() {
            cmd.arg(current_dir.join(path));
        } else {
            cmd.arg(arg);
        }
    }
    cmd.stdout(InheritFd(1));
    cmd.stderr(InheritFd(2));

    // Create temporary directory
    let temp_dir = TempDir::new("rust").unwrap();
    let temp_dir_path = temp_dir.path();
    let temp_dir_display = temp_dir_path.display();

    // Compile
    info!("cwd: {} | cmd: `{}`", temp_dir_display, cmd);
    match cmd.cwd(temp_dir_path).output() {
        Err(e) => panic!("`{}` failed: {}", cmd, e),
        Ok(ProcessOutput { status: exit, .. }) => if !exit.success() {
            let exit_code = match exit {
                ExitSignal(code) => code,
                ExitStatus(code) => code,
            };

            os::set_exit_status(exit_code);
            return;
        },
    }

    // Look for the produced binary
    let mut cmd = match fs::readdir(temp_dir_path) {
        Err(e) => panic!("`ls {}` failed: {}", temp_dir_display, e),
        Ok(paths) => match paths.as_slice().get(0) {
            Some(path) => Command::new(path),
            None => panic!("no binary found in {}", temp_dir_display),
        }
    };

    // Build the executable command
    match executable_args {
        None => {},
        Some(args) => {
            cmd.args(args);
        },
    }
    cmd.stdin(InheritFd(0));
    cmd.stdout(InheritFd(1));
    cmd.stderr(InheritFd(2));

    // Execute
    info!("cwd: . | cmd: `{}`", cmd);
    match cmd.output() {
        Err(e) => panic!("`{}` failed: {}", cmd, e),
        Ok(ProcessOutput { status: exit, .. }) => if !exit.success() {
            let exit_code = match exit {
                ExitSignal(code) => code,
                ExitStatus(code) => code,
            };

            os::set_exit_status(exit_code);
            return;
        },
    }
}
