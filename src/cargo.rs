//! Cargo

use std::env;
use std::fs::{File, self};
use std::io::{Read, Write, self};
use std::os::unix::fs::MetadataExt;
use std::os::unix;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use sha1::Sha1;

use Error;

/// A cargo project
pub struct Project {
    name: String,
    path: PathBuf
}

impl Project {
    /// Creates/updates a cargo project
    ///
    /// - The name of the cargo project will be derived from the input `source` file
    /// - The project will be located in the user cache directory
    pub fn new(source: PathBuf) -> Result<Project, Error> {
        // Prefixed debug message
        macro_rules! _debug {
            ($template:expr, $($args:expr),*) => {
                debug!(concat!("Project::new: ", $template), $($args),*)
            }
        }

        debug!("Project::new({:?})", source);

        let source = try!(source.canonicalize());
        debug!("Project::new: canonical path: {:?}", source);

        if !source.is_file() {
            _debug!("{:?} is not a file", source);
            return Err(Error::NotAFile(source))
        }

        let mut path_digest = Sha1::new();
        path_digest.update(
            source.parent()
            .map(|p| p.to_str().unwrap())
            .unwrap_or("./")
            .as_bytes()
        );
        let path_digest = path_digest.hexdigest()[0..16].to_string();

        // TODO "escape" `file_stem` to improve its chances to be a valid project name
        let name = source.file_stem().unwrap().to_str().unwrap();
        _debug!("using {:?} as project name", &name);

        let entry_name = format!("{}-{}", path_digest, &name);
        _debug!("using {:?} as cache entry name", &entry_name);

        let project = Project {
            path: try!(cache_dir()).join(entry_name),
            name: name.to_string()
        };

        {
            let path = project.path();
            let name = project.name();

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
                        .args(&["new", "--bin", "--name", name])
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
    pub fn modified(&self) -> io::Result<i64> {
        // Prefixed debug message
        macro_rules! _debug {
            ($template:expr, $($args:expr),*) => {
                debug!(concat!("Project::modified: ", $template), $($args),*)
            }
        }

        _debug!("opening `src/main.rs`",);
        let main = self.path().join("src/main.rs");
        let modified = try!(try!(File::open(main)).metadata()).mtime();
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
    pub fn run(&self, args: Vec<&str>) -> io::Result<Output> {
        let name = self.name();
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
        let ref mut f = try!(File::open(self.path.join("src/main.rs")));
        let ref mut text = String::new();
        try!(f.read_to_string(text));

        let mut start = false;
        for line in text.lines() {
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
    pub fn timestamp(&self) -> Result<Option<i64>, Error> {
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

    fn name(&self) -> &str {
        &self.name
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
