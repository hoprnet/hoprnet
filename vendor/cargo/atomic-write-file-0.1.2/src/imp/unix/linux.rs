use crate::imp::unix::copy_file_perms;
use crate::imp::unix::create_temporary_file;
use crate::imp::unix::remove_temporary_file;
use crate::imp::unix::rename_temporary_file;
use crate::imp::unix::Dir;
use crate::imp::unix::OpenOptions;
use crate::imp::unix::RandomName;
use nix::errno::Errno;
use nix::fcntl::openat;
use nix::fcntl::OFlag;
use nix::libc;
use nix::sys::stat::Mode;
use nix::unistd::linkat;
use nix::unistd::LinkatFlags;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs::File;
use std::io::Result;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::path::Path;

fn create_unnamed_temporary_file(dir: &Dir, opts: &OpenOptions) -> nix::Result<File> {
    let access_mode = if opts.read {
        OFlag::O_RDWR
    } else {
        OFlag::O_WRONLY
    };
    let flags = OFlag::O_TMPFILE
        | access_mode
        | OFlag::O_CLOEXEC
        | OFlag::from_bits_truncate(opts.custom_flags & !libc::O_ACCMODE);
    let create_mode = Mode::from_bits_truncate(opts.mode);

    let file_fd = openat(dir.as_raw_fd(), ".", flags, create_mode)?;

    let file = unsafe { File::from_raw_fd(file_fd) };
    Ok(file)
}

fn rename_unnamed_temporary_file(dir: &Dir, file: &File, name: &OsStr) -> nix::Result<()> {
    let fd = file.as_raw_fd();
    let src = OsString::from(format!("/proc/self/fd/{fd}"));
    let mut random_name = RandomName::new(name);

    let temporary_name = loop {
        match linkat(
            Some(dir.as_raw_fd()),
            src.as_os_str(),
            Some(dir.as_raw_fd()),
            random_name.next(),
            LinkatFlags::SymlinkFollow,
        ) {
            Ok(()) => break random_name.into_os_string(),
            Err(Errno::EEXIST) => continue,
            Err(err) => return Err(err),
        }
    };

    rename_temporary_file(dir, &temporary_name, name)
}

#[derive(Debug)]
pub(crate) struct TemporaryFile {
    pub(crate) dir: Dir,
    pub(crate) file: File,
    pub(crate) name: OsString,
    pub(crate) temporary_name: Option<OsString>,
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

        // Try to open an unnamed temporary file (with O_TMPFILE). This may not be supported on all
        // filesystems; if it's not supported, create a named temporary file in the same way the
        // generic Unix implementation would do.
        let (file, temporary_name) = match create_unnamed_temporary_file(&dir, opts) {
            Ok(file) => (file, None),
            // Linux >= 3.11 may return ENOTSUP if the filesystem does not support unnamed
            // temporary files; Linux < 3.11 will return EISDIR because O_TMPFILE is not supported
            // at all.
            Err(Errno::ENOTSUP) | Err(Errno::EISDIR) => {
                let (file, temporary_name) = create_temporary_file(&dir, opts, &name)?;
                (file, Some(temporary_name))
            }
            Err(err) => return Err(err.into()),
        };

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
        match self.temporary_name {
            None => rename_unnamed_temporary_file(&self.dir, &self.file, &self.name)?,
            Some(ref temporary_name) => {
                rename_temporary_file(&self.dir, temporary_name, &self.name)?
            }
        }
        Ok(())
    }

    pub(crate) fn remove_file(&self) -> Result<()> {
        match self.temporary_name {
            None => (),
            Some(ref temporary_name) => remove_temporary_file(&self.dir, temporary_name)?,
        }
        Ok(())
    }
}
