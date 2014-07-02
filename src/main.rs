#![crate_id = "rustic"]
#![feature(phase)]

#[phase(plugin, link)] extern crate log;

use std::io::fs;
use std::io::process::{Command,ExitSignal,ExitStatus};
use std::os;

use tmpdir::TmpDir;

mod child;
mod rand;
mod tmpdir;

// FIXME Need a better way to detect which argument is the crate file
fn is_crate(arg: &str) -> bool {
    arg.ends_with(".rs")
}

fn main() {
    let args = os::args();
    let args = args.slice_from(1);

    // If `--run` is not in the arguments: pass all the arguments to `rustc`
    if args.iter().all(|arg| arg.as_slice() != "--run") {
        let mut cmd = Command::new("rustc");
        cmd.args(args);

        info!("cwd: . | cmd: `{}`", cmd);
        // FIXME Replace `output` for `spawn`, and redirect child std{out,err}
        match cmd.spawn() {
            Err(e) => fail!("`{}` failed: {}", cmd, e),
            Ok(p) => match child::supplant(p) {
                Err(e) => fail!("`{}` failed: {}", cmd, e),
                Ok(exit) => {
                    if !exit.success() {
                        let exit_code = match exit {
                            ExitSignal(code) => code,
                            ExitStatus(code) => code,
                        };

                        os::set_exit_status(exit_code);
                        return;
                    }
                },
            },
        }
    }

    // Separate the crate file from the other arguments
    let current_dir = os::getcwd();
    let crate_path = match args.iter().find(|arg| is_crate(arg.as_slice())) {
        Some(arg) => {
            let path = Path::new(arg.as_slice());
            if path.is_absolute() {
                path
            } else {
                current_dir.join(path)
            }
        }
        None => fail!("Didn't find a crate file in the arguments passed: {}",
                      args),
    };
    let args: Vec<&str> = args.iter().filter_map(|arg| {
        let arg = arg.as_slice();
        if is_crate(arg) { None } else { Some(arg) }
    }).collect();

    let tmpdir = TmpDir::new();
    let tmpdir_path = tmpdir.path();
    let tmpdir_display = tmpdir_path.display();

    // Before `--run`: arguments for the compiler
    let mut splitted_args = args.as_slice().split(|&flag| flag == "--run");
    let rustc_args = splitted_args.next().unwrap();

    // TODO `--crate-type=lib` should also be forbidden
    // XXX What if the crate has a `crate_type=*lib` attribute?
    if rustc_args.iter().any(|&flag| flag == "--out-dir") {
        fail!("Can't use both `--out-dir` and `--run` flags at the same time");
    }

    // Compile
    let mut cmd = Command::new("rustc");
    cmd.args(rustc_args).arg(crate_path);
    info!("cwd: {} | cmd: `{}`", tmpdir_display, cmd);
    // FIXME Replace `output` for `spawn`, and redirect child std{out,err}
    match cmd.cwd(tmpdir_path).spawn() {
        Err(e) => fail!("`{}` failed: {}", cmd, e),
        Ok(p) => match child::supplant(p) {
            Err(e) => fail!("`{}` failed: {}", cmd, e),
            Ok(exit) => {
                if !exit.success() {
                    let exit_code = match exit {
                        ExitSignal(code) => code,
                        ExitStatus(code) => code,
                    };

                    os::set_exit_status(exit_code);
                    return;
                }
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

    // After `--run`: arguments for the executable
    let executable_args = splitted_args.next().unwrap();
    cmd.args(executable_args);

    // Execute
    info!("cwd: . | cmd: `{}`", cmd);
    // FIXME Replace `output` for `spawn`, and redirect child std{out,err}
    match cmd.spawn() {
        Err(e) => fail!("`{}` failed: {}", cmd, e),
        Ok(p) => match child::supplant(p) {
            Err(e) => fail!("`{}` failed: {}", cmd, e),
            Ok(exit) => {
                if !exit.success() {
                    let exit_code = match exit {
                        ExitSignal(code) => code,
                        ExitStatus(code) => code,
                    };

                    os::set_exit_status(exit_code);
                    return;
                }
            },
        }
    }
}
