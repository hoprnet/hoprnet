use rand::distributions::Alphanumeric;
use rand::Rng;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::path::Path;
use std::path::PathBuf;

fn is_a_directory() -> Error {
    // TODO Use `ErrorKind::IsADirectory` once it is stabilized
    Error::new(ErrorKind::InvalidInput, "path is a directory")
}

#[derive(Clone, Debug)]
pub(crate) struct OpenOptions {
    pub(crate) read: bool,
}

impl OpenOptions {
    pub(crate) fn new() -> Self {
        Self { read: false }
    }
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub(crate) struct TemporaryFile {
    pub(crate) temp_path: PathBuf,
    pub(crate) dest_path: PathBuf,
    pub(crate) file: File,
}

impl TemporaryFile {
    pub(crate) fn open(opts: &OpenOptions, path: &Path) -> Result<Self> {
        let dest_path = path.to_owned();
        let dir_path = dest_path.parent().ok_or_else(is_a_directory)?;
        let name = dest_path
            .file_name()
            .ok_or_else(is_a_directory)?
            .to_os_string();

        let mut random_name = RandomName::new(&name);
        let (file, temp_path) = loop {
            let path = dir_path.join(random_name.next());
            match File::options()
                .write(true)
                .read(opts.read)
                .create_new(true)
                .open(&path)
            {
                Ok(file) => break (file, path),
                Err(ref err) if err.kind() == ErrorKind::AlreadyExists => continue,
                Err(err) => return Err(err),
            }
        };

        Ok(Self {
            temp_path,
            dest_path,
            file,
        })
    }

    pub(crate) fn rename_file(&self) -> Result<()> {
        fs::rename(&self.temp_path, &self.dest_path)
    }

    pub(crate) fn remove_file(&self) -> Result<()> {
        fs::remove_file(&self.temp_path)
    }
}

struct RandomName<'a> {
    base_name: &'a OsStr,
}

impl<'a> RandomName<'a> {
    const SUFFIX_SIZE: usize = 6;

    fn new(base_name: &'a OsStr) -> Self {
        Self { base_name }
    }

    fn next(&mut self) -> OsString {
        let mut rng = rand::thread_rng();
        let mut name = OsString::with_capacity(1 + self.base_name.len() + 1 + Self::SUFFIX_SIZE);
        let mut suffix = Vec::with_capacity(Self::SUFFIX_SIZE);
        name.push(".");
        name.push(self.base_name);
        name.push(".");
        for _ in 0..Self::SUFFIX_SIZE {
            suffix.push(rng.sample(Alphanumeric));
        }
        // SAFETY: `suffix` contains only ASCII alphanumeric characters, which are valid utf-8
        // characters
        name.push(unsafe { String::from_utf8_unchecked(suffix) });
        name
    }
}
