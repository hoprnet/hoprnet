//! Unix-specific extensions to [`AtomicWriteFile`] and [`OpenOptions`].

use crate::imp::Preserve;
use crate::AtomicWriteFile;
use crate::OpenOptions;
use nix::sys::stat::mode_t;
use std::io::Result;
use std::os::unix::fs;

#[cfg(feature = "unstable-unix_file_vectored_at")]
use std::io::IoSlice;

#[cfg(feature = "unstable-unix_file_vectored_at")]
use std::io::IoSliceMut;

impl fs::OpenOptionsExt for OpenOptions {
    #[inline]
    fn mode(&mut self, mode: u32) -> &mut Self {
        self.inner.mode = mode as mode_t;
        self
    }

    #[inline]
    fn custom_flags(&mut self, flags: i32) -> &mut Self {
        self.inner.custom_flags = flags;
        self
    }
}

/// Unix-specific extensions to [`OpenOptions`].
///
/// On Unix, [`AtomicWriteFile`] works by creating a temporary file, and then renaming the file to
/// its final name at commit time. If a file with the same name already exists, it is often
/// desiderable that the new file written by [`AtomicWriteFile`] preserves the same permissions,
/// attributes, and other metadata as the original file. This extension trait allows you to control
/// which metadata should be preserved, and which metadata should use default values instead.
///
/// **Note:** [`AtomicWriteFile`] samples and copies the metadata from the original file (if any)
/// to the temporary file when the [`AtomicWriteFile`] is initially opened. If the original file
/// metadata is changed, or the original file is deleted, or the original file gets (re-)created,
/// before the [`AtomicWriteFile`] is committed, those changes won't be reflected in the final
/// result.
pub trait OpenOptionsExt {
    /// Specifies whether the atomically-written file should have the file permissions of the
    /// original file (if any).
    ///
    /// If `true` (the default), the file permissions of the original file (if any) are copied over
    /// to the atomically-written file when [`OpenOptions::open()`] is called.
    ///
    /// If `false`, or if no original file exists when [`OpenOptions::open()`] is called, the
    /// default mode `0o666` is used, which will be masked with the process umask. The default mask
    /// can be customized using [`std::os::unix::fs::OpenOptionsExt::mode()`].
    ///
    /// This method only preserves the permissions that can be set through `chmod(2)`. This method
    /// has no effect on ACLs (POSIX Access Control Lists).
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// # let test_dir = option_env!("TEST_DIR").unwrap_or("target/test-files");
    /// # std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
    /// # std::env::set_current_dir(test_dir).expect("failed to move to test dir");
    /// use atomic_write_file::OpenOptions;
    /// use atomic_write_file::unix::OpenOptionsExt;
    ///
    /// let mut options = OpenOptions::new();
    /// options.preserve_mode(false);
    /// let file = options.open("foo.txt")?;
    /// file.commit()?; // "foo.txt" is saved with the default mode
    /// # Ok(())
    /// # }
    /// ```
    fn preserve_mode(&mut self, preserve_mode: bool) -> &mut Self;

    /// Specifies whether the atomically-written file should have the same ownership (user/group)
    /// of the original file (if any).
    ///
    /// If `true`, the ownership of the original file (if any) is copied over
    /// to the atomically-written file when [`OpenOptions::open()`] is called.
    ///
    /// If `false`, or if no original file exists when [`OpenOptions::open()`] is called, the
    /// ownership is set using the default platform-specific semantics.
    ///
    /// **Note:** setting ownership of files generally requires root privileges on Unix platforms
    /// (or `CAP_CHOWN` on Linux). As such, using `preserve_owner(true)` when the process is not
    /// running as root may result in [`OpenOptions::open()`] to fail with an error.
    ///
    /// The related method [`OpenOptionsExt::try_preserve_owner()`] allows to preserve ownership
    /// only if the process has sufficient privileges. It is a variant of this option that does not
    /// cause [`OpenOptions::open()`] to fail when the process does not have enough privileges to
    /// preserve ownership.
    ///
    /// By default, `try_preserve_owner(true)` is used.
    ///
    /// Calling `preserve_owner()` overrides any previous call to `preserve_owner()` or
    /// `try_preserve_owner()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// # let test_dir = option_env!("TEST_DIR").unwrap_or("target/test-files");
    /// # std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
    /// # std::env::set_current_dir(test_dir).expect("failed to move to test dir");
    /// use atomic_write_file::OpenOptions;
    /// use atomic_write_file::unix::OpenOptionsExt;
    ///
    /// let mut options = OpenOptions::new();
    /// options.preserve_owner(true);
    /// let file = options.open("foo.txt")?; // this fails if "foo.txt" exists and its ownership
    ///                                      // cannot be preserved
    /// file.commit()?; // "foo.txt" is saved with the same ownership as the original "foo.txt"
    ///                 // (if any)
    /// # Ok(())
    /// # }
    /// ```
    fn preserve_owner(&mut self, preserve_owner: bool) -> &mut Self;

    /// Specifies whether the atomically-written file should have the ownership information
    /// (user/group) of the original file (if any). If ownership information cannot be set on the
    /// atomically-written file, and if the process is not running as root, then
    /// [`OpenOptions::open()`] silently continues suppressing the error.
    ///
    /// Using `try_preserve_owner(true)` is equivalent to using
    /// [`OpenOptionsExt::preserve_owner(true)`](OpenOptionsExt::preserve_owner), with the
    /// exception that this option does not cause [`OpenOptions::open()`] to fail with a "Operation
    /// not permitted" (`EPERM`) error if the process is not running as root (effective UID 0).
    ///
    /// Note that [`OpenOptions::open()`] may still fail with an error when setting ownership if an
    /// error other than "Operation not permitted" (`EPERM`) is thrown.
    ///
    /// If the process is running as root, then all errors are reported by [`OpenOptions::open()`].
    ///
    /// If `false`, or if no original file exists when [`OpenOptions::open()`] is called, the
    /// ownership is set using the default platform-specific semantics.
    ///
    /// The default value for this option is `true`.
    ///
    /// Calling `try_preserve_owner()` overrides any previous call to `preserve_owner()` or
    /// `try_preserve_owner()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// # let test_dir = option_env!("TEST_DIR").unwrap_or("target/test-files");
    /// # std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
    /// # std::env::set_current_dir(test_dir).expect("failed to move to test dir");
    /// use atomic_write_file::OpenOptions;
    /// use atomic_write_file::unix::OpenOptionsExt;
    ///
    /// let mut options = OpenOptions::new();
    /// options.try_preserve_owner(true); // this is the default
    /// let file = options.open("foo.txt")?; // this won't fail if "foo.txt" exists, its ownership
    ///                                      // cannot be preserved, and the process is not running
    ///                                      // as root
    /// file.commit()?; // "foo.txt" is saved with the same ownership as the original "foo.txt"
    ///                 // (if any)
    /// # Ok(())
    /// # }
    /// ```
    fn try_preserve_owner(&mut self, try_preserve_owner: bool) -> &mut Self;
}

impl OpenOptionsExt for OpenOptions {
    #[inline]
    fn preserve_mode(&mut self, preserve_mode: bool) -> &mut Self {
        self.inner.preserve_mode = preserve_mode;
        self
    }

    #[inline]
    fn preserve_owner(&mut self, preserve_owner: bool) -> &mut Self {
        self.inner.preserve_owner = match preserve_owner {
            true => Preserve::Yes,
            false => Preserve::No,
        };
        self
    }

    #[inline]
    fn try_preserve_owner(&mut self, try_preserve_owner: bool) -> &mut Self {
        self.inner.preserve_owner = match try_preserve_owner {
            true => Preserve::Try,
            false => Preserve::No,
        };
        self
    }
}

impl fs::FileExt for AtomicWriteFile {
    #[inline]
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        self.temporary_file.file.read_at(buf, offset)
    }
    #[inline]
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        self.temporary_file.file.write_at(buf, offset)
    }

    #[inline]
    #[cfg(feature = "unstable-unix_file_vectored_at")]
    fn read_vectored_at(&self, bufs: &mut [IoSliceMut<'_>], offset: u64) -> Result<usize> {
        self.temporary_file.file.read_vectored_at(bufs, offset)
    }

    #[inline]
    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> Result<()> {
        self.temporary_file.file.read_exact_at(buf, offset)
    }

    #[inline]
    #[cfg(feature = "unstable-unix_file_vectored_at")]
    fn write_vectored_at(&self, bufs: &[IoSlice<'_>], offset: u64) -> Result<usize> {
        self.temporary_file.file.write_vectored_at(bufs, offset)
    }

    #[inline]
    fn write_all_at(&self, buf: &[u8], offset: u64) -> Result<()> {
        self.temporary_file.file.write_all_at(buf, offset)
    }
}
