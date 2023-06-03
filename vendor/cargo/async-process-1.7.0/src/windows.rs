//! Windows-specific extensions.

#[cfg(not(async_process_no_windows_raw_arg))]
use std::ffi::OsStr;
use std::os::windows::io::{AsRawHandle, RawHandle};
use std::os::windows::process::CommandExt as _;

use crate::{Child, Command};

/// Windows-specific extensions to the [`Command`] builder.
///
/// This trait is sealed: it cannot be implemented outside `async-process`.
/// This is so that future additional methods are not breaking changes.
pub trait CommandExt: crate::sealed::Sealed {
    /// Sets the [process creation flags][1] to be passed to `CreateProcess`.
    ///
    /// These will always be ORed with `CREATE_UNICODE_ENVIRONMENT`.
    ///
    /// [1]: https://docs.microsoft.com/en-us/windows/win32/procthread/process-creation-flags
    fn creation_flags(&mut self, flags: u32) -> &mut Command;

    /// Append literal text to the command line without any quoting or escaping.
    ///
    /// This is useful for passing arguments to `cmd.exe /c`, which doesn't follow
    /// `CommandLineToArgvW` escaping rules.
    ///
    /// **Note:** This method is only available on Rust 1.62+.
    #[cfg(not(async_process_no_windows_raw_arg))]
    fn raw_arg<S: AsRef<OsStr>>(&mut self, text_to_append_as_is: S) -> &mut Command;
}

impl crate::sealed::Sealed for Command {}
impl CommandExt for Command {
    fn creation_flags(&mut self, flags: u32) -> &mut Command {
        self.inner.creation_flags(flags);
        self
    }

    #[cfg(not(async_process_no_windows_raw_arg))]
    fn raw_arg<S: AsRef<OsStr>>(&mut self, text_to_append_as_is: S) -> &mut Command {
        self.inner.raw_arg(text_to_append_as_is);
        self
    }
}

impl AsRawHandle for Child {
    fn as_raw_handle(&self) -> RawHandle {
        self.child.lock().unwrap().get_mut().as_raw_handle()
    }
}
