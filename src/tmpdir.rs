use rand::RandomSequences;
use std::io::{fs,UserDir};
use std::os;

pub struct TmpDir {
    path: Path,
}

impl TmpDir {
    pub fn new() -> TmpDir {
        let temp = os::tmpdir();
        let path =
            RandomSequences::new().map(|s| temp.join(format!("rust-{}", s)))
                                  .filter(|p| !p.exists())
                                  .next().unwrap();

        info!("`mkdir {}`", path.display());
        match fs::mkdir(&path, UserDir) {
            Err(e) => fail!("`mkdir {}` failed: {}", path.display(), e),
            Ok(_) => {},
        }

        TmpDir {
            path: path,
        }
    }

    pub fn path<'a>(&'a self) -> &'a Path {
        &self.path
    }
}

impl Drop for TmpDir {
    fn drop(&mut self) {
        let display = self.path.display();

        info!("`rm -rf {}`", display);
        match fs::rmdir_recursive(&self.path) {
            Err(e) => println!("`rm -rf {} failed`: {}", display, e),
            Ok(_) => {},
        }
    }
}
