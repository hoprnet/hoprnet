//! is-terminal is a simple utility that answers one question:
//!
//! > Is this a terminal?
//!
//! A "terminal", also known as a "tty", is an I/O device which may be
//! interactive and may support color and other special features. This crate
//! doesn't provide any of those features; it just answers this one question.
//!
//! On Unix-family platforms, this is effectively the same as the [`isatty`]
//! function for testing whether a given stream is a terminal, though it
//! accepts high-level stream types instead of raw file descriptors.
//!
//! On Windows, it uses a variety of techniques to determine whether the
//! given stream is a terminal.
//!
//! # Example
//!
//! ```rust
//! use is_terminal::IsTerminal;
//!
//! if std::io::stdout().is_terminal() {
//!     println!("stdout is a terminal")
//! }
//! ```
//!
//! [`isatty`]: https://man7.org/linux/man-pages/man3/isatty.3.html

#![cfg_attr(unix, no_std)]

#[cfg(not(target_os = "unknown"))]
use io_lifetimes::AsFilelike;
#[cfg(windows)]
use io_lifetimes::BorrowedHandle;
#[cfg(windows)]
use windows_sys::Win32::System::Console::STD_HANDLE;

pub trait IsTerminal {
    /// Returns true if this is a terminal.
    ///
    /// # Example
    ///
    /// ```
    /// use is_terminal::IsTerminal;
    ///
    /// if std::io::stdout().is_terminal() {
    ///     println!("stdout is a terminal")
    /// }
    /// ```
    fn is_terminal(&self) -> bool;
}

#[cfg(not(target_os = "unknown"))]
impl<Stream: AsFilelike> IsTerminal for Stream {
    #[inline]
    fn is_terminal(&self) -> bool {
        #[cfg(any(unix, target_os = "wasi"))]
        {
            rustix::termios::isatty(self)
        }

        #[cfg(target_os = "hermit")]
        {
            hermit_abi::isatty(self.as_filelike().as_fd())
        }

        #[cfg(windows)]
        {
            _is_terminal(self.as_filelike())
        }
    }
}

// The Windows implementation here is copied from atty, with #51 and #54
// applied. The only significant modification is to take a `BorrowedHandle`
// argument instead of using a `Stream` enum.

#[cfg(windows)]
fn _is_terminal(stream: BorrowedHandle<'_>) -> bool {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::Console::GetStdHandle;
    use windows_sys::Win32::System::Console::{
        STD_ERROR_HANDLE as STD_ERROR, STD_INPUT_HANDLE as STD_INPUT,
        STD_OUTPUT_HANDLE as STD_OUTPUT,
    };

    let (fd, others) = unsafe {
        if stream.as_raw_handle() == GetStdHandle(STD_INPUT) as _ {
            (STD_INPUT, [STD_ERROR, STD_OUTPUT])
        } else if stream.as_raw_handle() == GetStdHandle(STD_OUTPUT) as _ {
            (STD_OUTPUT, [STD_INPUT, STD_ERROR])
        } else if stream.as_raw_handle() == GetStdHandle(STD_ERROR) as _ {
            (STD_ERROR, [STD_INPUT, STD_OUTPUT])
        } else {
            return false;
        }
    };
    if unsafe { console_on_any(&[fd]) } {
        // False positives aren't possible. If we got a console then
        // we definitely have a tty on stdin.
        return true;
    }

    // At this point, we *could* have a false negative. We can determine that
    // this is true negative if we can detect the presence of a console on
    // any of the other streams. If another stream has a console, then we know
    // we're in a Windows console and can therefore trust the negative.
    if unsafe { console_on_any(&others) } {
        return false;
    }

    // Otherwise, we fall back to a very strange msys hack to see if we can
    // sneakily detect the presence of a tty.
    unsafe { msys_tty_on(fd) }
}

/// Returns true if any of the given fds are on a console.
#[cfg(windows)]
unsafe fn console_on_any(fds: &[STD_HANDLE]) -> bool {
    use windows_sys::Win32::System::Console::{GetConsoleMode, GetStdHandle};

    for &fd in fds {
        let mut out = 0;
        let handle = GetStdHandle(fd);
        if GetConsoleMode(handle, &mut out) != 0 {
            return true;
        }
    }
    false
}

/// Returns true if there is an MSYS tty on the given handle.
#[cfg(windows)]
unsafe fn msys_tty_on(fd: STD_HANDLE) -> bool {
    use std::ffi::c_void;
    use windows_sys::Win32::{
        Foundation::MAX_PATH,
        Storage::FileSystem::{FileNameInfo, GetFileInformationByHandleEx},
        System::Console::GetStdHandle,
    };

    /// Mirrors windows_sys::Win32::Storage::FileSystem::FILE_NAME_INFO, giving
    /// it a fixed length that we can stack allocate
    #[repr(C)]
    #[allow(non_snake_case)]
    struct FILE_NAME_INFO {
        FileNameLength: u32,
        FileName: [u16; MAX_PATH as usize],
    }
    let mut name_info = FILE_NAME_INFO {
        FileNameLength: 0,
        FileName: [0; MAX_PATH as usize],
    };
    // Safety: function has no invariants. an invalid handle id will cause
    //         GetFileInformationByHandleEx to return an error
    let handle = GetStdHandle(fd);
    // Safety: handle is valid, and buffer length is fixed
    let res = GetFileInformationByHandleEx(
        handle,
        FileNameInfo,
        &mut name_info as *mut _ as *mut c_void,
        std::mem::size_of::<FILE_NAME_INFO>() as u32,
    );
    if res == 0 {
        return false;
    }
    let s = &name_info.FileName[..name_info.FileNameLength as usize];
    let name = String::from_utf16_lossy(s);
    // This checks whether 'pty' exists in the file name, which indicates that
    // a pseudo-terminal is attached. To mitigate against false positives
    // (e.g., an actual file name that contains 'pty'), we also require that
    // either the strings 'msys-' or 'cygwin-' are in the file name as well.)
    let is_msys = name.contains("msys-") || name.contains("cygwin-");
    let is_pty = name.contains("-pty");
    is_msys && is_pty
}

#[cfg(target_os = "unknown")]
impl IsTerminal for std::io::Stdin {
    #[inline]
    fn is_terminal(&self) -> bool {
        false
    }
}

#[cfg(target_os = "unknown")]
impl IsTerminal for std::io::Stdout {
    #[inline]
    fn is_terminal(&self) -> bool {
        false
    }
}

#[cfg(target_os = "unknown")]
impl IsTerminal for std::io::Stderr {
    #[inline]
    fn is_terminal(&self) -> bool {
        false
    }
}

#[cfg(target_os = "unknown")]
impl<'a> IsTerminal for std::io::StdinLock<'a> {
    #[inline]
    fn is_terminal(&self) -> bool {
        false
    }
}

#[cfg(target_os = "unknown")]
impl<'a> IsTerminal for std::io::StdoutLock<'a> {
    #[inline]
    fn is_terminal(&self) -> bool {
        false
    }
}

#[cfg(target_os = "unknown")]
impl<'a> IsTerminal for std::io::StderrLock<'a> {
    #[inline]
    fn is_terminal(&self) -> bool {
        false
    }
}

#[cfg(target_os = "unknown")]
impl<'a> IsTerminal for std::fs::File {
    #[inline]
    fn is_terminal(&self) -> bool {
        false
    }
}

#[cfg(target_os = "unknown")]
impl IsTerminal for std::process::ChildStdin {
    #[inline]
    fn is_terminal(&self) -> bool {
        false
    }
}

#[cfg(target_os = "unknown")]
impl IsTerminal for std::process::ChildStdout {
    #[inline]
    fn is_terminal(&self) -> bool {
        false
    }
}

#[cfg(target_os = "unknown")]
impl IsTerminal for std::process::ChildStderr {
    #[inline]
    fn is_terminal(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_os = "unknown"))]
    use super::IsTerminal;

    #[test]
    #[cfg(windows)]
    fn stdin() {
        assert_eq!(
            atty::is(atty::Stream::Stdin),
            std::io::stdin().is_terminal()
        )
    }

    #[test]
    #[cfg(windows)]
    fn stdout() {
        assert_eq!(
            atty::is(atty::Stream::Stdout),
            std::io::stdout().is_terminal()
        )
    }

    #[test]
    #[cfg(windows)]
    fn stderr() {
        assert_eq!(
            atty::is(atty::Stream::Stderr),
            std::io::stderr().is_terminal()
        )
    }

    #[test]
    #[cfg(any(unix, target_os = "wasi"))]
    fn stdin() {
        unsafe {
            assert_eq!(
                atty::is(atty::Stream::Stdin),
                rustix::io::stdin().is_terminal()
            )
        }
    }

    #[test]
    #[cfg(any(unix, target_os = "wasi"))]
    fn stdout() {
        unsafe {
            assert_eq!(
                atty::is(atty::Stream::Stdout),
                rustix::io::stdout().is_terminal()
            )
        }
    }

    #[test]
    #[cfg(any(unix, target_os = "wasi"))]
    fn stderr() {
        unsafe {
            assert_eq!(
                atty::is(atty::Stream::Stderr),
                rustix::io::stderr().is_terminal()
            )
        }
    }

    #[test]
    #[cfg(any(unix, target_os = "wasi"))]
    fn stdin_vs_libc() {
        unsafe {
            assert_eq!(
                libc::isatty(libc::STDIN_FILENO) != 0,
                rustix::io::stdin().is_terminal()
            )
        }
    }

    #[test]
    #[cfg(any(unix, target_os = "wasi"))]
    fn stdout_vs_libc() {
        unsafe {
            assert_eq!(
                libc::isatty(libc::STDOUT_FILENO) != 0,
                rustix::io::stdout().is_terminal()
            )
        }
    }

    #[test]
    #[cfg(any(unix, target_os = "wasi"))]
    fn stderr_vs_libc() {
        unsafe {
            assert_eq!(
                libc::isatty(libc::STDERR_FILENO) != 0,
                rustix::io::stderr().is_terminal()
            )
        }
    }
}
