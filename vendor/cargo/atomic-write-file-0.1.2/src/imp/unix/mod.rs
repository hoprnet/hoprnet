use nix::errno::Errno;
use nix::fcntl::open;
use nix::fcntl::openat;
use nix::fcntl::renameat;
use nix::fcntl::AtFlags;
use nix::fcntl::OFlag;
use nix::libc;
use nix::sys::stat::fchmod;
use nix::sys::stat::fstatat;
use nix::sys::stat::mode_t;
use nix::sys::stat::Mode;
use nix::unistd::fchown;
use nix::unistd::unlinkat;
use nix::unistd::Gid;
use nix::unistd::Uid;
use nix::unistd::UnlinkatFlags;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs::File;
use std::io::Result;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::os::fd::RawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::ffi::OsStringExt;
use std::path::Path;

#[cfg(all(target_os = "linux", feature = "unnamed-tmpfile"))]
mod linux;

#[cfg(all(target_os = "linux", feature = "unnamed-tmpfile"))]
pub(crate) use self::linux::*;

#[cfg(not(all(target_os = "linux", feature = "unnamed-tmpfile")))]
mod generic;

#[cfg(not(all(target_os = "linux", feature = "unnamed-tmpfile")))]
pub(crate) use self::generic::*;

#[derive(Copy, Clone, Debug)]
pub(crate) enum Preserve {
    No,
    Yes,
    Try,
}

impl Preserve {
    fn is_yes(&self) -> bool {
        match self {
            Self::No => false,
            Self::Yes | Self::Try => true,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct OpenOptions {
    pub(crate) read: bool,
    pub(crate) mode: mode_t,
    pub(crate) custom_flags: i32,
    pub(crate) preserve_mode: bool,
    pub(crate) preserve_owner: Preserve,
}

impl OpenOptions {
    pub(crate) fn new() -> Self {
        Self {
            read: false,
            mode: 0o666,
            custom_flags: 0,
            preserve_mode: true,
            preserve_owner: Preserve::Try,
        }
    }
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub(crate) struct Dir {
    fd: OwnedFd,
}

impl Dir {
    pub(crate) fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let fd = open(
            path.as_ref(),
            OFlag::O_DIRECTORY | OFlag::O_CLOEXEC,
            Mode::empty(),
        )?;
        Ok(unsafe { Self::from_raw_fd(fd) })
    }
}

impl AsRawFd for Dir {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl FromRawFd for Dir {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self {
            fd: OwnedFd::from_raw_fd(fd),
        }
    }
}

struct RandomName {
    buf: Vec<u8>,
}

impl RandomName {
    const SUFFIX_SIZE: usize = 6;

    fn new(base_name: &OsStr) -> Self {
        let buf_len = 1 + base_name.len() + 1 + Self::SUFFIX_SIZE;
        let mut buf = Vec::with_capacity(buf_len);
        buf.push(b'.');
        buf.extend_from_slice(base_name.as_bytes());
        buf.push(b'.');
        buf.extend_from_slice(&[0; Self::SUFFIX_SIZE]);
        debug_assert_eq!(buf_len, buf.len());
        Self { buf }
    }

    fn next(&mut self) -> &OsStr {
        let mut rng = rand::thread_rng();
        let buf_len = self.buf.len();
        let suffix = &mut self.buf[buf_len - RandomName::SUFFIX_SIZE..];
        for c in suffix.iter_mut() {
            *c = rng.sample(Alphanumeric);
        }
        OsStr::from_bytes(&self.buf)
    }

    #[inline]
    fn into_os_string(self) -> OsString {
        OsString::from_vec(self.buf)
    }
}

fn create_temporary_file(
    dir: &Dir,
    opts: &OpenOptions,
    name: &OsStr,
) -> nix::Result<(File, OsString)> {
    let access_mode = if opts.read {
        OFlag::O_RDWR
    } else {
        OFlag::O_WRONLY
    };
    let flags = access_mode
        | OFlag::O_CREAT
        | OFlag::O_EXCL
        | OFlag::O_CLOEXEC
        | OFlag::from_bits_truncate(opts.custom_flags & !libc::O_ACCMODE);
    let create_mode = Mode::from_bits_truncate(opts.mode);

    let mut random_name = RandomName::new(name);
    let file_fd = loop {
        match openat(dir.as_raw_fd(), random_name.next(), flags, create_mode) {
            Ok(file_fd) => break file_fd,
            Err(Errno::EEXIST) => continue,
            Err(err) => return Err(err),
        }
    };

    let file = unsafe { File::from_raw_fd(file_fd) };
    let temporary_name = random_name.into_os_string();
    Ok((file, temporary_name))
}

fn rename_temporary_file(dir: &Dir, temporary_name: &OsStr, name: &OsStr) -> nix::Result<()> {
    renameat(
        Some(dir.as_raw_fd()),
        temporary_name,
        Some(dir.as_raw_fd()),
        name,
    )
}

fn remove_temporary_file(dir: &Dir, temporary_name: &OsStr) -> nix::Result<()> {
    unlinkat(
        Some(dir.as_raw_fd()),
        temporary_name,
        UnlinkatFlags::NoRemoveDir,
    )
}

fn maybe_ignore_eperm(result: nix::Result<()>, preserve: Preserve) -> nix::Result<()> {
    match result {
        Err(Errno::EPERM) => match preserve {
            Preserve::Try => {
                if Uid::effective().is_root() {
                    result
                } else {
                    Ok(())
                }
            }
            _ => result,
        },
        _ => result,
    }
}

fn copy_file_perms<P: AsRef<Path>>(
    dir: &Dir,
    copy_from: P,
    copy_to: &File,
    opts: &OpenOptions,
) -> Result<()> {
    let stat = match fstatat(
        dir.as_raw_fd(),
        copy_from.as_ref(),
        AtFlags::AT_SYMLINK_NOFOLLOW,
    ) {
        Ok(stat) => stat,
        Err(Errno::ENOENT) => return Ok(()),
        Err(err) => return Err(err.into()),
    };
    if opts.preserve_mode {
        let mode = Mode::from_bits_retain(stat.st_mode);
        fchmod(copy_to.as_raw_fd(), mode)?;
    }
    if opts.preserve_owner.is_yes() {
        let uid = Uid::from_raw(stat.st_uid);
        let gid = Gid::from_raw(stat.st_gid);
        maybe_ignore_eperm(
            fchown(copy_to.as_raw_fd(), Some(uid), Some(gid)),
            opts.preserve_owner,
        )?;
    }
    Ok(())
}
