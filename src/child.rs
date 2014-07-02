use std::io::{BufferedReader,IoResult};
use std::io::process::{Process,ProcessExit};
use std::io::stdio;

// "Redirect" child process stdout/stderr to `rustic` stdout/stderr
// XXX The "redirection" flushes on a line by line basis, if the child process
// flushes its stdout/stderr, `rustic` will ignore it. :-(
pub fn supplant(mut p: Process) -> IoResult<ProcessExit> {
    let p_stdout = p.stdout.take_unwrap();
    spawn(proc() {
        let mut stdout = stdio::stdout();

        for line in BufferedReader::new(p_stdout).lines() {
            stdout.write_str(line.unwrap().as_slice());
        }
    });

    let p_stderr = p.stderr.take_unwrap();
    spawn(proc() {
        let mut stderr = stdio::stderr();

        for line in BufferedReader::new(p_stderr).lines() {
            stderr.write_str(line.unwrap().as_slice());
        }
    });

    p.wait()
}
