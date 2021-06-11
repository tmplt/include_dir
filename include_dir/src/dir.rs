use crate::file::File;
use std::fs;
use std::io::Write;
use std::path::Path;

/// A directory entry.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Dir<'a> {
    #[doc(hidden)]
    pub path: &'a str,
    #[doc(hidden)]
    pub files: &'a [File<'a>],
    #[doc(hidden)]
    pub dirs: &'a [Dir<'a>],
}

impl<'a> Dir<'a> {
    /// Get the directory's path.
    pub fn path(&self) -> &'a Path {
        Path::new(self.path)
    }

    /// Get a list of the files in this directory.
    pub fn files(&self) -> &'a [File<'a>] {
        self.files
    }

    /// Get a list of the sub-directories inside this directory.
    pub fn dirs(&self) -> &'a [Dir<'a>] {
        self.dirs
    }

    /// Does this directory contain `path`?
    pub fn contains<S: AsRef<Path>>(&self, path: S) -> bool {
        let path = path.as_ref();

        self.get_file(path).is_some() || self.get_dir(path).is_some()
    }

    /// Fetch a sub-directory by *exactly* matching its path relative to the
    /// directory included with `include_dir!()`.
    pub fn get_dir<S: AsRef<Path>>(&self, path: S) -> Option<Dir<'a>> {
        let path = path.as_ref();

        for dir in self.dirs {
            if Path::new(dir.path) == path {
                return Some(*dir);
            }

            if let Some(d) = dir.get_dir(path) {
                return Some(d);
            }
        }

        None
    }

    /// Fetch a sub-directory by *exactly* matching its path relative to the
    /// directory included with `include_dir!()`.
    pub fn get_file<S: AsRef<Path>>(&self, path: S) -> Option<File<'a>> {
        let path = path.as_ref();

        for file in self.files {
            if Path::new(file.path) == path {
                return Some(*file);
            }
        }

        for dir in self.dirs {
            if let Some(d) = dir.get_file(path) {
                return Some(d);
            }
        }

        None
    }

    /// Create directories and extract all files to real filesystem.
    /// Creates parent directories of `path` if they do not already exist.
    /// Fails if some files already exist.
    /// In case of error, partially extracted directory may remain on the filesystem.
    pub fn extract<S: AsRef<Path>>(&self, path: S) -> std::io::Result<()> {
        let path = path.as_ref();

        // create directories first
        for dir in self.dirs() {
            fs::create_dir_all(path.join(dir.path()))?;
        }

        for file in self
            .dirs()
            .iter()
            .flat_map(|d| d.files())
            .chain(self.files())
        {
            let mut fsf = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path.join(file.path()))?;
            fsf.write_all(file.contents())?;
            fsf.sync_all()?;
        }

        Ok(())
    }
}
