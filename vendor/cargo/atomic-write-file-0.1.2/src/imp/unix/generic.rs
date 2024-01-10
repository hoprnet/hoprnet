use crate::imp::unix::copy_file_perms;
use crate::imp::unix::create_temporary_file;
use crate::imp::unix::remove_temporary_file;
use crate::imp::unix::rename_temporary_file;
use crate::imp::unix::Dir;
use crate::imp::unix::OpenOptions;
use nix::errno::Errno;
use std::ffi::OsString;
use std::fs::File;
use std::io::Result;
use std::path::Path;

#[derive(Debug)]
pub(crate) struct TemporaryFile {
    pub(crate) dir: Dir,
    pub(crate) file: File,
    pub(crate) name: OsString,
    pub(crate) temporary_name: OsString,
}

impl TemporaryFile {
    pub(crate) fn open(opts: &OpenOptions, path: &Path) -> Result<Self> {
        let dir_path = path.parent().ok_or(Errno::EISDIR)?;
        let name = path.file_name().ok_or(Errno::EISDIR)?.to_os_string();

        let dir = if !dir_path.as_os_str().is_empty() {
            Dir::open(dir_path)?
        } else {
            Dir::open(".")?
        };

        let (file, temporary_name) = create_temporary_file(&dir, opts, &name)?;

        if opts.preserve_mode || opts.preserve_owner.is_yes() {
            copy_file_perms(&dir, &name, &file, opts)?;
        }

        Ok(Self {
            dir,
            file,
            name,
            temporary_name,
        })
    }

    pub(crate) fn rename_file(&self) -> Result<()> {
        rename_temporary_file(&self.dir, &self.temporary_name, &self.name)?;
        Ok(())
    }

    pub(crate) fn remove_file(&self) -> Result<()> {
        remove_temporary_file(&self.dir, &self.temporary_name)?;
        Ok(())
    }
}
