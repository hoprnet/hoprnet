//! Redox-specific system library.
#![no_std]

pub use syscall::error;
use syscall::error::{Error, Result};

pub mod flag {
    pub use libc::{O_CREAT, O_DIRECTORY, O_CLOEXEC, O_RDONLY, O_RDWR, O_WRONLY, O_ACCMODE, O_NONBLOCK};

    pub use libc::{CLOCK_MONOTONIC, CLOCK_REALTIME};

    pub use libc::{SIG_BLOCK, SIG_UNBLOCK, SIG_SETMASK};

    pub use libc::{SIGUSR1, SIGUSR2};

    #[cfg(target_os = "redox")]
    pub use libc::{O_SHLOCK, O_EXLOCK};

    #[cfg(target_os = "redox")]
    pub const O_STAT: i32 = syscall::flag::O_STAT as i32;
}

pub mod errno {
    pub use libc::{EPERM, EACCES, EBADF, EBADFD, ENODEV, ESRCH, EPIPE, ESPIPE, EBUSY, EINTR, EAGAIN, EWOULDBLOCK};
}
pub mod data {
    pub use libc::timespec as TimeSpec;
    pub use libc::sigaction as SigAction;

    #[cfg(target_os = "redox")]
    pub use libc::sigset_t as SigSet;

    // TODO: Should libredox compile on non-Redox platforms?
    #[cfg(not(target_os = "redox"))]
    pub type SigSet = u64;
}

type RawResult = usize;

extern "C" {
    // NOTE: Although there are version suffixes, there'd have to be strong reasons for adding new
    // version.
    fn redox_open_v1(path_base: *const u8, path_len: usize, flags: u32, mode: u16) -> RawResult;
    fn redox_dup_v1(fd: usize, buf: *const u8, len: usize) -> RawResult;
    fn redox_dup2_v1(old_fd: usize, new_fd: usize, buf: *const u8, len: usize) -> RawResult;
    fn redox_read_v1(fd: usize, dst_base: *mut u8, dst_len: usize) -> RawResult;
    fn redox_write_v1(fd: usize, src_base: *const u8, src_len: usize) -> RawResult;
    fn redox_fsync_v1(fd: usize) -> RawResult;
    fn redox_fdatasync_v1(fd: usize) -> RawResult;
    fn redox_fchmod_v1(fd: usize, new_mode: u16) -> RawResult;
    fn redox_fchown_v1(fd: usize, new_uid: u32, new_gid: u32) -> RawResult;
    fn redox_fpath_v1(fd: usize, dst_base: *mut u8, dst_len: usize) -> RawResult;
    fn redox_close_v1(fd: usize) -> RawResult;

    // NOTE: While the Redox kernel currently doesn't distinguish between threads and processes,
    // the return value of this function is expected to be treated as a process ID and not a thread
    // ID.
    fn redox_get_pid_v1() -> RawResult;

    fn redox_get_euid_v1() -> RawResult;
    fn redox_get_ruid_v1() -> RawResult;
    fn redox_get_egid_v1() -> RawResult;
    fn redox_get_rgid_v1() -> RawResult;
    fn redox_setrens_v1(rns: usize, ens: usize) -> RawResult;

    fn redox_kill_v1(pid: usize, signal: u32) -> RawResult;
    fn redox_waitpid_v1(pid: usize, status: *mut i32, options: u32) -> RawResult;

    fn redox_sigprocmask_v1(how: u32, new: *const u64, old: *mut u64) -> RawResult;
    fn redox_sigaction_v1(signal: u32, new: *const data::SigAction, old: *mut data::SigAction) -> RawResult;

    fn redox_clock_gettime_v1(clock: usize, ts: *mut data::TimeSpec) -> RawResult;
}

#[cfg(feature = "call")]
pub struct Fd(usize);

#[cfg(feature = "call")]
impl Fd {
    #[inline]
    pub fn open(path: &str, flags: i32, mode: u16) -> Result<Self> {
        Ok(Self(call::open(path, flags, mode)?))
    }
    #[inline]
    pub fn dup(&self, buf: &[u8]) -> Result<usize> {
        call::dup(self.raw(), buf)
    }
    #[inline]
    pub fn dup2(&self, new_fd: usize, buf: &[u8]) -> Result<usize> {
        call::dup2(self.raw(), new_fd, buf)
    }

    #[inline]
    pub const fn raw(&self) -> usize { self.0 }

    #[inline]
    pub fn into_raw(self) -> usize {
        let raw = self.raw();
        core::mem::forget(self);
        raw
    }

    #[inline]
    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        call::read(self.raw(), buf)
    }
    #[inline]
    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        call::write(self.raw(), buf)
    }
    #[inline]
    pub fn fpath(&self, path: &mut [u8]) -> Result<usize> {
        call::fpath(self.raw(), path)
    }
    #[inline]
    pub fn fsync(&self) -> Result<()> {
        call::fsync(self.raw())
    }
    #[inline]
    pub fn fdatasync(&self) -> Result<()> {
        call::fdatasync(self.raw())
    }

    #[inline]
    pub fn chmod(&self, new_mode: u16) -> Result<()> {
        call::fchmod(self.raw(), new_mode)
    }
    #[inline]
    pub fn chown(&self, new_uid: u32, new_gid: u32) -> Result<()> {
        call::fchown(self.raw(), new_uid, new_gid)
    }
    #[inline]
    pub fn close(self) -> Result<()> {
        call::close(self.into_raw())
    }
}
#[cfg(feature = "call")]
impl Drop for Fd {
    fn drop(&mut self) {
        let _ = unsafe { redox_close_v1(self.0) };
    }
}

#[cfg(feature = "call")]
pub mod call {
    use super::*;

    /// flags and mode are binary compatible with libc
    #[inline]
    pub fn open(path: impl AsRef<str>, flags: i32, mode: u16) -> Result<usize> {
        let path = path.as_ref();
        Ok(Error::demux(unsafe { redox_open_v1(path.as_ptr(), path.len(), flags as u32, mode) })?)
    }
    #[inline]
    pub fn dup(fd: usize, buf: impl AsRef<[u8]>) -> Result<usize> {
        let buf = buf.as_ref();
        Ok(Error::demux(unsafe { redox_dup_v1(fd, buf.as_ptr(), buf.len()) })?)
    }
    #[inline]
    pub fn dup2(old_fd: usize, new_fd: usize, buf: impl AsRef<[u8]>) -> Result<usize> {
        let buf = buf.as_ref();
        Ok(Error::demux(unsafe { redox_dup2_v1(old_fd, new_fd, buf.as_ptr(), buf.len()) })?)
    }
    #[inline]
    pub fn read(raw_fd: usize, buf: &mut [u8]) -> Result<usize> {
        Ok(Error::demux(unsafe { redox_read_v1(raw_fd, buf.as_mut_ptr(), buf.len()) })?)
    }
    #[inline]
    pub fn write(raw_fd: usize, buf: &[u8]) -> Result<usize> {
        Ok(Error::demux(unsafe { redox_write_v1(raw_fd, buf.as_ptr(), buf.len()) })?)
    }
    #[inline]
    pub fn fsync(raw_fd: usize) -> Result<()> {
        Error::demux(unsafe { redox_fsync_v1(raw_fd) }).map(|_| ())
    }
    #[inline]
    pub fn fdatasync(raw_fd: usize) -> Result<()> {
        Ok(Error::demux(unsafe { redox_fdatasync_v1(raw_fd) })?).map(|_| ())
    }
    #[inline]
    pub fn fchmod(raw_fd: usize, new_mode: u16) -> Result<()> {
        Error::demux(unsafe { redox_fchmod_v1(raw_fd, new_mode) })?;
        Ok(())
    }
    #[inline]
    pub fn fchown(raw_fd: usize, new_uid: u32, new_gid: u32) -> Result<()> {
        Error::demux(unsafe { redox_fchown_v1(raw_fd, new_uid, new_gid) })?;
        Ok(())
    }
    #[inline]
    pub fn fpath(raw_fd: usize, buf: &mut [u8]) -> Result<usize> {
        Ok(Error::demux(unsafe { redox_fpath_v1(raw_fd, buf.as_mut_ptr(), buf.len()) })?)
    }
    #[inline]
    pub fn close(raw_fd: usize) -> Result<()> {
        Error::demux(unsafe { redox_close_v1(raw_fd) })?;
        Ok(())
    }

    #[inline]
    pub fn geteuid() -> Result<usize> {
        Error::demux(unsafe { redox_get_euid_v1() })
    }

    #[inline]
    pub fn getruid() -> Result<usize> {
        Error::demux(unsafe { redox_get_ruid_v1() })
    }

    #[inline]
    pub fn getegid() -> Result<usize> {
        Error::demux(unsafe { redox_get_egid_v1() })
    }
    #[inline]
    pub fn getrgid() -> Result<usize> {
        Error::demux(unsafe { redox_get_rgid_v1() })
    }
    #[inline]
    pub fn getpid() -> Result<usize> {
        Error::demux(unsafe { redox_get_pid_v1() })
    }
    #[inline]
    pub fn setrens(rns: usize, ens: usize) -> Result<usize> {
        Error::demux(unsafe { redox_setrens_v1(rns, ens) })
    }
    #[inline]
    pub fn waitpid(pid: usize, status: &mut i32, options: i32) -> Result<usize> {
        Error::demux(unsafe { redox_waitpid_v1(pid, status as *mut i32, options as u32) })
    }
    #[inline]
    pub fn kill(pid: usize, signal: u32) -> Result<()> {
        Error::demux(unsafe { redox_kill_v1(pid, signal) }).map(|_| ())
    }
    #[inline]
    pub fn clock_gettime(clock: i32, ts: &mut data::TimeSpec) -> Result<()> {
        Error::demux(unsafe { redox_clock_gettime_v1(clock as usize, ts) }).map(|_| ())
    }
    #[inline]
    pub fn sigprocmask(how: i32, newmask: Option<&data::SigSet>, oldmask: Option<&mut data::SigSet>) -> Result<()> {
        Error::demux(unsafe { redox_sigprocmask_v1(how as u32, newmask.map_or(core::ptr::null(), |m| m), oldmask.map_or(core::ptr::null_mut(), |m| m)) }).map(|_| ())
    }
    #[inline]
    pub fn sigaction(signal: i32, newact: Option<&data::SigAction>, oldact: Option<&mut data::SigAction>) -> Result<()> {
        Error::demux(unsafe { redox_sigaction_v1(signal as u32, newact.map_or(core::ptr::null(), |m| m), oldact.map_or(core::ptr::null_mut(), |m| m)) }).map(|_| ())
    }
}

#[cfg(feature = "scheme")]
pub mod scheme {
    use super::*;

    use core::mem::size_of;

    pub use syscall::{Scheme, SchemeMut, SchemeBlock, SchemeBlockMut};
    use syscall::Packet;

    #[repr(transparent)]
    #[derive(Clone, Copy, Default)]
    pub struct Request(Packet);

    impl Request {
        pub fn handle_scheme(mut self, scheme: &impl Scheme) -> Response {
            scheme.handle(&mut self.0);
            Response(self.0)
        }
        pub fn handle_scheme_mut(mut self, scheme: &mut impl SchemeMut) -> Response {
            scheme.handle(&mut self.0);
            Response(self.0)
        }
        pub fn handle_scheme_block(mut self, scheme: &mut impl SchemeBlock) -> Result<Response, Request> {
            match scheme.handle(&mut self.0) {
                Some(code) => Ok(Response(Packet { a: code, ..self.0 })),
                None => Err(self),
            }
        }
        pub fn handle_scheme_block_mut(mut self, scheme: &mut impl SchemeBlockMut) -> Result<Response, Request> {
            match scheme.handle(&mut self.0) {
                Some(code) => Ok(Response(Packet { a: code, ..self.0 })),
                None => Err(self),
            }
        }
    }

    #[repr(transparent)]
    #[derive(Clone, Copy, Default)]
    pub struct Response(Packet);

    impl Response {
        pub fn new(req: &Request, status: Result<usize>) -> Self {
            Self(Packet {
                a: Error::mux(status),
                ..req.0
            })
        }
    }

    pub enum SignalBehavior {
        Interrupt,
        Restart,
    }

    // TODO: Support uninitialized memory
    #[inline]
    pub fn read_requests(socket: usize, buf: &mut [Request], behavior: SignalBehavior) -> Result<usize> {
        let len = buf.len().checked_mul(size_of::<Request>()).unwrap();

        let bytes_read = loop {
            match call::read(socket, unsafe { core::slice::from_raw_parts_mut(buf.as_mut_ptr().cast(), len) }) {
                Ok(n) => break n,
                error @ Err(Error { errno: errno::EINTR }) => match behavior {
                    SignalBehavior::Restart => continue,
                    SignalBehavior::Interrupt => return error,
                }
                Err(err) => return Err(err),
            }
        };

        debug_assert_eq!(bytes_read % size_of::<Request>(), 0);

        Ok(bytes_read / size_of::<Request>())
    }

    #[inline]
    pub fn write_responses(socket: usize, buf: &[Response], behavior: SignalBehavior) -> Result<usize> {
        let bytes = unsafe { core::slice::from_raw_parts(buf.as_ptr().cast(), buf.len().checked_mul(size_of::<Response>()).unwrap()) };

        let bytes_written = loop {
            match call::write(socket, bytes) {
                Ok(n) => break n,
                error @ Err(Error { errno: errno::EINTR }) => match behavior {
                    SignalBehavior::Restart => continue,
                    SignalBehavior::Interrupt => return error,
                }
                Err(err) => return Err(err),
            }
        };
        debug_assert_eq!(bytes_written % size_of::<Response>(), 0);
        Ok(bytes_written / size_of::<Response>())
    }
}
