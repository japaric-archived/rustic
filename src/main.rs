#![crate_name = "rustic"]
#![feature(phase)]

#[phase(plugin, link)] extern crate log;

use std::io::fs;
use std::io::process::{Command,ExitSignal,ExitStatus};
use std::os;

use tmpdir::TmpDir;

mod child;
mod rand;
mod tmpdir;

fn is_crate(arg: &str) -> bool {
    Path::new(arg).exists()
}

fn main() {
    let args = os::args();
    let args = args.slice_from(1);

    // If `--run` is not in the arguments: pass all the arguments to `rustc`
    if args.iter().all(|arg| arg.as_slice() != "--run") {
        let mut cmd = Command::new("rustc");
        cmd.args(args);

        info!("cwd: . | cmd: `{}`", cmd);
        match cmd.spawn() {
            Err(e) => fail!("`{}` failed: {}", cmd, e),
            Ok(p) => match child::supplant(p) {
                Err(e) => fail!("`{}` failed: {}", cmd, e),
                Ok(exit) => if !exit.success() {
                    let exit_code = match exit {
                        ExitSignal(code) => code,
                        ExitStatus(code) => code,
                    };

                    os::set_exit_status(exit_code);
                    return;
                },
            },
        }
    }

    // Before `--run`: arguments for the compiler
    let mut splitted_args = args.split(|arg| arg.as_slice() == "--run");
    let rustc_args = splitted_args.next().unwrap();
    let executable_args = splitted_args.next().unwrap();

    // TODO `--crate-type=lib` should also be forbidden
    // XXX What if the crate has a `crate_type=*lib` attribute?
    if rustc_args.iter().any(|arg| arg.as_slice() == "--out-dir") {
        fail!("Can't use both `--out-dir` and `--run` flags at the same time");
    }

    // Separate the crate file from the other arguments
    let crate_arg = match args.iter().find(|arg| {
        is_crate(arg.as_slice())
    }) {
        Some(arg) => arg,
        None => fail!("Didn't find a crate file in the arguments passed"),
    };

    // Use full path for the crate file
    let current_dir = os::getcwd();
    let crate_path = {
        let path = Path::new(crate_arg.as_slice());

        if path.is_absolute() {
            path
        } else {
            current_dir.join(path)
        }
    };

    // Build the rustc command
    let mut cmd = Command::new("rustc");
    for arg in rustc_args.iter().filter(|&arg| arg != crate_arg) {
        cmd.arg(arg.as_slice());
    }
    cmd.arg(crate_path);

    // Create temporary directory
    let tmpdir = TmpDir::new();
    let tmpdir_path = tmpdir.path();
    let tmpdir_display = tmpdir_path.display();

    // Compile
    info!("cwd: {} | cmd: `{}`", tmpdir_display, cmd);
    match cmd.cwd(tmpdir_path).spawn() {
        Err(e) => fail!("`{}` failed: {}", cmd, e),
        Ok(p) => match child::supplant(p) {
            Err(e) => fail!("`{}` failed: {}", cmd, e),
            Ok(exit) => if !exit.success() {
                let exit_code = match exit {
                    ExitSignal(code) => code,
                    ExitStatus(code) => code,
                };

                os::set_exit_status(exit_code);
                return;
            },
        },
    }

    // Look for the produced binary
    let mut cmd = match fs::readdir(tmpdir_path) {
        Err(e) => fail!("`ls {}` failed: {}", tmpdir_display, e),
        Ok(paths) => match paths.as_slice().get(0) {
            Some(path) => Command::new(path),
            None => fail!("no binary found in {}", tmpdir_display),
        }
    };

    // Build the executable command
    for arg in executable_args.iter().filter(|&arg| arg != crate_arg) {
        cmd.arg(arg.as_slice());
    }

    // Execute
    info!("cwd: . | cmd: `{}`", cmd);
    match cmd.spawn() {
        Err(e) => fail!("`{}` failed: {}", cmd, e),
        Ok(p) => match child::supplant(p) {
            Err(e) => fail!("`{}` failed: {}", cmd, e),
            Ok(exit) => if !exit.success() {
                let exit_code = match exit {
                    ExitSignal(code) => code,
                    ExitStatus(code) => code,
                };

                os::set_exit_status(exit_code);
                return;
            },
        }
    }
}
