//! Cargo

use std::env;
use std::ffi::OsString;
use std::fs::{File, PathExt, self};
use std::io::{BufReader, Read, Write, self};
use std::os::unix;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use lines::Lines;

use Error;

/// A cargo project
pub struct Project {
    path: PathBuf,
}

impl Project {
    /// Creates/updates a cargo project
    ///
    /// - The name of the cargo project will be derived from the input `source` file
    /// - The project will be located in the user cache directory
    pub fn new(mut source: PathBuf) -> Result<Project, Error> {
        // Prefixed debug message
        macro_rules! _debug {
            ($template:expr, $($args:expr),*) => {
                debug!(concat!("Project::new: ", $template), $($args),*)
            }
        }

        debug!("Project::new({:?})", source);

        if source.is_relative() {
            source = try!(env::current_dir()).join(source);
            _debug!("using absolute path: {:?}", source);
        }

        if !source.is_file() {
            _debug!("{:?} is not a file", source);
            return Err(Error::NotAFile(source))
        }

        // TODO "escape" `file_stem` to improve its chances to be a valid project name
        let name = source.file_stem().unwrap_or_else(|| unsafe {
            debug_unreachable!();
        });

        _debug!("using {:?} as project name", name);
        let project = Project {
            path: try!(cache_dir()).join(name),
        };

        {
            let path = project.path();
            let replace_main = || -> Result<(), Error> {
                _debug!("removing old `src/main.rs` file",);
                try!(fs::remove_file(path.join("src/main.rs")));
                _debug!("OK",);

                _debug!("updating `src/main.rs` symlink",);
                try!(unix::fs::symlink(&source, path.join("src/main.rs")));
                _debug!("OK",);

                Ok(())
            };

            if !path.exists() {
                _debug!("project {:?} doesn't exist, creating a new one in {:?}", name, path);
                let output = try! {
                    Command::new("cargo")
                        .args(&["new", "--bin"])
                        .arg(&path)
                        .output()
                };

                if !output.status.success() {
                    _debug!("`cargo new` failed",);
                    return Err(Error::CargoNew(output.stderr))
                }
                _debug!("OK",);

                _debug!("backing up original `Cargo.toml`",);
                try!(fs::copy(path.join("Cargo.toml"), path.join("Cargo.toml.orig")));
                _debug!("OK",);

                try!(replace_main());

                _debug!("updating `Cargo.toml`",);
                try!(project.update_cargo_toml());
                _debug!("OK",);

                _debug!("building project",);
                try!(project.build());
                _debug!("OK",);

                _debug!("updating time stamp",);
                try!(project.update_timestamp());
                _debug!("OK",);
            } else {
                _debug!("project {:?} already exists", name);
                try!(replace_main());
            }
        }

        Ok(project)
    }

    /// Builds the project if the source file has changed
    pub fn build(&self) -> Result<(), Error> {
        // Prefixed debug message
        macro_rules! _debug {
            ($template:expr, $($args:expr),*) => {
                debug!(concat!("Project::build: ", $template), $($args),*)
            }
        }

        let output = try! {
            Command::new("cargo")
                .args(&["build", "--release"])
                .current_dir(self.path())
                .output()
        };

        if !output.status.success() {
            _debug!("FAILED",);

            return Err(Error::CargoBuild(output.stderr))
        }

        _debug!("OK",);

        Ok(())
    }

    /// Checks if the project has changed since the last build
    pub fn has_changed(&self) -> Result<bool, Error> {
        // Prefixed debug message
        macro_rules! _debug {
            ($template:expr, $($args:expr),*) => {
                debug!(concat!("Project::modified: ", $template), $($args),*)
            }
        }

        let modified = try!(self.modified());
        let timestamp = try!(self.timestamp());

        if let Some(timestamp) = timestamp {
            if modified > timestamp {
                _debug!("YES, last modified on {}, timestamp was {}", modified, timestamp);

                Ok(true)
            } else {
                _debug!("NO, last modified on {}, timestamp was {}", modified, timestamp);

                Ok(false)
            }
        } else {
            _debug!("YES, `time.stamp` doesn't exist",);

            Ok(true)
        }
    }

    /// Returns the current modified time
    pub fn modified(&self) -> io::Result<u64> {
        // Prefixed debug message
        macro_rules! _debug {
            ($template:expr, $($args:expr),*) => {
                debug!(concat!("Project::modified: ", $template), $($args),*)
            }
        }

        _debug!("opening `src/main.rs`",);
        let main = self.path().join("src/main.rs");
        let modified = try!(try!(File::open(main)).metadata()).modified();
        _debug!("OK",);

        _debug!("`src/main.rs` was last modified on {}", modified);
        Ok(modified)
    }

    /// Removes `Cargo.lock`
    pub fn remove_lock(&self) -> io::Result<()> {
        // Prefixed debug message
        macro_rules! _debug {
            ($template:expr, $($args:expr),*) => {
                debug!(concat!("Project::remove_lock: ", $template), $($args),*)
            }
        }

        let lock_file = self.path().join("Cargo.lock");

        if lock_file.exists() {
            _debug!("removing `Cargo.lock`",);
            try!(fs::remove_file(lock_file));
            _debug!("OK",);
        } else {
            _debug!("`Cargo.lock` doesn't exist",);
        }


        Ok(())
    }

    /// Executes the binary
    pub fn run<I>(&self, args: I) -> io::Result<Output> where I: Iterator<Item=OsString> {
        let name = self.path().file_stem().unwrap_or_else(|| unsafe {
            debug_unreachable!()
        });

        let executable = self.path().join("target/release").join(name);

        let mut cmd = Command::new(executable);

        for arg in args {
            cmd.arg(arg);
        }

        cmd.stderr(Stdio::inherit());
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());

        debug!("Project::run: `{:?}`", cmd);
        cmd.output()
    }

    /// Extracts a `Cargo.toml` from the source file comments and updates the project `Cargo.toml`
    pub fn update_cargo_toml(&self) -> io::Result<()> {
        // Prefixed debug message
        macro_rules! _debug {
            ($template:expr, $($args:expr),*) => {
                debug!(concat!("Project::update_cargo_toml: ", $template), $($args),*)
            }
        }

        _debug!("reading original `Cargo.toml`",);
        let mut string = String::new();
        try!(try!(File::open(self.path.join("Cargo.toml.orig"))).read_to_string(&mut string));
        _debug!("OK",);

        _debug!("reading `src/main.rs` comments",);
        let f = try!(File::open(self.path.join("src/main.rs")));
        let mut lines = Lines::from(BufReader::new(f));

        let mut start = false;
        while let Some(line) = lines.next() {
            let line = try!(line);

            if start {
                if line.starts_with("//") {
                    string.push_str(line["//".len()..].trim());
                    string.push('\n');
                } else {
                    break
                }
            } else if line.starts_with("// Cargo.toml") {
                start = true;
            }
        }
        _debug!("final `Cargo.toml`: {:?}", string);

        _debug!("writing new `Cargo.toml`",);
        try!(try!(File::create(self.path().join("Cargo.toml"))).write_all(string.as_bytes()));
        _debug!("OK",);

        Ok(())
    }

    /// Updates `time.stamp` with current modified time
    pub fn update_timestamp(&self) -> io::Result<()> {
        // Prefixed debug message
        macro_rules! _debug {
            ($template:expr, $($args:expr),*) => {
                debug!(concat!("Project::update_timestamp: ", $template), $($args),*)
            }
        }

        let timestamp = try!(self.modified()).to_string();

        _debug!("writing {} into `time.stamp`", timestamp);
        try!(try!(File::create(self.path().join("time.stamp"))).write_all(timestamp.as_bytes()));
        _debug!("OK",);

        Ok(())
    }

    /// Reads `time.stamp` and returns its value
    pub fn timestamp(&self) -> Result<Option<u64>, Error> {
        // Prefixed debug message
        macro_rules! _debug {
            ($template:expr, $($args:expr),*) => {
                debug!(concat!("Project::timestamp: ", $template), $($args),*)
            }
        }

        let stamp_file = self.path().join("time.stamp");

        if !stamp_file.exists() {
            _debug!("`time.stamp` doesn't exist",);
            return Ok(None)
        }

        _debug!("reading `time.stamp`",);
        let mut string = String::new();
        try!(try!(File::open(&stamp_file)).read_to_string(&mut string));
        _debug!("OK",);

        if let Ok(stamp) = string.parse() {
            _debug!("got {}", stamp);
            Ok(Some(stamp))
        } else {
            _debug!("malformed timestamp, removing `time.stamp`",);
            try!(fs::remove_file(&stamp_file));
            _debug!("OK",);

            Err(Error::MalformedTimestamp(string))
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

/// Returns the absolute path to the cache directory
fn cache_dir() -> Result<PathBuf, Error> {
    if let Some(cache_dir) = env::var_os("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(cache_dir));
    }

    if let Some(home_dir) = env::var_os("HOME") {
        let mut path = PathBuf::from(home_dir);
        path.push(".cache/rustic");

        return Ok(path);
    }

    Err(Error::NoCacheDir)
}
