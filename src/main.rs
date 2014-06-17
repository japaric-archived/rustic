#![crate_id = "rustic"]
#![feature(phase)]

#[phase(plugin, link)] extern crate log;

use std::io::fs;
use std::io::process::{Command,ExitSignal,ExitStatus,ProcessOutput};
use std::os;
use std::str;
use tmpdir::TmpDir;

mod rand;
mod tmpdir;

fn main() {
    let args = os::args();
    let args = args.slice_from(1);

    // No `--run` flag, pass all the flags to `rustc`
    if !args.iter().any(|a| a.as_slice() == "--run") {
        let mut cmd = Command::new("rustc");
        cmd.args(args);

        info!("cwd: . | cmd: `{}`", cmd);
        match cmd.output() {
            Err(e) => fail!("`{}`: {}", cmd, e),
            Ok(ProcessOutput { output: out, error: err, status: exit }) => {
                match exit {
                    ExitSignal(exit) => os::set_exit_status(exit),
                    ExitStatus(exit) => os::set_exit_status(exit),
                }

                print!("{}", str::from_utf8_lossy(out.as_slice()));
                print!("{}", str::from_utf8_lossy(err.as_slice()));

                return;
            },
        }
    }

    // TODO `--crate-type=lib` should also be forbidden
    // XXX What if the crate has a `crate_type=*lib` attribute?
    if args.iter().any(|f| f.as_slice() == "--out-dir") {
        fail!("Can't use both `--out-dir and `--run` flags at the same time");
    }

    let cwd = os::getcwd();
    let tmpdir = TmpDir::new();
    let mut bench = false;
    let mut cmd = Command::new("rustc");
    for arg in args.iter().filter(|a| a.as_slice() != "--run") {
        // FIXME Need a better way to detect which argument is the source file
        let arg = arg.as_slice();

        if arg == "--bench" {
            bench = true;
            cmd.arg("--test");
        } else if arg.ends_with(".rs") {
            // Use full path in the source file
            cmd.arg(cwd.join(arg.as_slice()));
        } else {
            cmd.arg(arg);
        }
    }
    cmd.cwd(tmpdir.path());

    // Compile
    info!("cwd: {} | cmd: `{}`", tmpdir.path().display(), cmd);
    match cmd.output() {
        Err(e) => fail!("`{}`: {}", cmd, e),
        Ok(ProcessOutput { output: out, error: err, status: exit }) => {
            let exit_code = match exit {
                ExitSignal(code) => code,
                ExitStatus(code) => code,
            };

            print!("{}", str::from_utf8_lossy(out.as_slice()));
            print!("{}", str::from_utf8_lossy(err.as_slice()));

            if !exit.success() {
                os::set_exit_status(exit_code);
                return;
            }
        },
    }

    // Execute
    let mut cmd = match fs::readdir(tmpdir.path()) {
        Err(e) => fail!("`ls {}`: {}", tmpdir.path().display(), e),
        Ok(paths) => Command::new(paths.get(0)),
    };

    if bench {
        cmd.arg("--bench");
    }

    info!("cwd: . | cmd: `{}`", cmd);
    match cmd.output() {
        Err(e) => fail!("`{}`: {}", cmd, e),
        Ok(ProcessOutput { output: out, error: err, status: exit }) => {
            let exit_code = match exit {
                ExitSignal(code) => code,
                ExitStatus(code) => code,
            };

            print!("{}", str::from_utf8_lossy(out.as_slice()));
            print!("{}", str::from_utf8_lossy(err.as_slice()));

            os::set_exit_status(exit_code);
        }
    }
}
