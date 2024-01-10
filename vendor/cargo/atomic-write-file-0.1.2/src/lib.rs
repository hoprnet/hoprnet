//! # Atomic Write File
//!
//! This crate offers functionality to write and overwrite files *atomically*, that is: without
//! leaving the file in an intermediate state. Either the new contents of the files are written to
//! the filesystem, or the old contents (if any) are preserved.
//!
//! This crate implements two main structs: [`AtomicWriteFile`] and [`OpenOptions`], which mimic
//! the standard [`std::fs::File`] and [`std::fs::OpenOptions`] as much as possible.
//!
//! This crate supports all major platforms, including: Unix systems, Windows, and WASI.
//!
//! ## Motivation and Example
//!
//! Consider the following snippet of code to write a configuration file in JSON format:
//!
//! ```
//! # fn main() -> std::io::Result<()> {
//! # let test_dir = option_env!("TEST_DIR").unwrap_or("target/test-files");
//! # std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
//! # std::env::set_current_dir(test_dir).expect("failed to move to test dir");
//! use std::io::Write;
//! use std::fs::File;
//!
//! let mut file = File::options()
//!                     .write(true)
//!                     .create(true)
//!                     .open("config.json")?;
//!
//! writeln!(file, "{{")?;
//! writeln!(file, "  \"key1\": \"value1\",")?;
//! writeln!(file, "  \"key2\": \"value2\"")?;
//! writeln!(file, "}}")?;
//! # Ok(())
//! # }
//! ```
//!
//! This code opens a file named `config.json`, truncates its contents (if the file already
//! existed), and writes the JSON content line-by-line.
//!
//! If the code is interrupted before all of the `writeln!` calls are completed (because of a
//! panic, or a signal is received, or the process is killed, or a filesystem error occurs), then
//! the file will be left in a broken state: it will not contain valid JSON data, and the original
//! contents (if any) will be lost.
//!
//! [`AtomicWriteFile`] solves this problem by placing the new contents into the destination file
//! only after it has been completely written to the filesystem. The snippet above can be rewritten
//! using [`AtomicWriteFile`] instead of [`File`] as follows:
//!
//! ```
//! # fn main() -> std::io::Result<()> {
//! # let test_dir = option_env!("TEST_DIR").unwrap_or("target/test-files");
//! # std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
//! # std::env::set_current_dir(test_dir).expect("failed to move to test dir");
//! use std::io::Write;
//! use atomic_write_file::AtomicWriteFile;
//!
//! let mut file = AtomicWriteFile::options()
//!                                .open("config.json")?;
//!
//! writeln!(file, "{{")?;
//! writeln!(file, "  \"key1\": \"value1\",")?;
//! writeln!(file, "  \"key2\": \"value2\"")?;
//! writeln!(file, "}}")?;
//!
//! file.commit()?;
//! # Ok(())
//! # }
//! ```
//!
//! Note that this code is almost the same as the original, except that it now uses
//! `AtomicWriteFile` instead of `File` and there's an additional call to `commit()`.
//!
//! If the code is interrupted early, before the call to `commit()`, the original file
//! `config.json` will be left untouched. Only if the new contents are fully written to the
//! filesystem, `config.json` will get them.
//!
//! ## How it works
//!
//! This crate works by creating a temporary file in the same directory as the destination file,
//! and then replacing the destination file with the temporary file once the new contents are fully
//! written to the filesystem.
//!
//! On **Unix**, the implementation is roughly equivalent to this pseudocode:
//!
//! ```text
//! fd = open("/path/to/directory/.filename.XXXXXX", O_WRONLY | O_CLOEXEC);
//! /* ... write contents ... */
//! fsync(fd);
//! rename("/path/to/directory/.filename.XXXXXX", "/path/to/directory/filename");
//! ```
//!
//! Where `XXXXXX` represents a random suffix. On **non-Unix** platforms, the implementation is
//! similar and uses the equivalent platform-specific system calls.
//!
//! On **Unix**, the actual implementation is more robust and makes use of directory file
//! descriptors (and the system calls `openat`, `linkat`, `renameat`) to make sure that, if the
//! directory is renamed or remounted during the operations, the file still ends up in the original
//! destination directory, and no cross-device writes happen.
//!
//! On **Linux**, the implementation makes use of anonymous temporary files (opened with
//! [`O_TMPFILE`](https://www.man7.org/linux/man-pages/man2/open.2.html)) if supported, and the
//! implementation is roughly equivalent to this pseudocode:
//!
//! ```text
//! fd = open("/path/to/directory", O_TMPFILE | O_WRONLY | O_CLOEXEC);
//! /* ... write contents ... */
//! fsync(fd);
//! link("/proc/self/fd/$fd", "/path/to/directory/.filename.XXXXXX");
//! rename("/path/to/directory/.filename.XXXXXX", "/path/to/directory/filename");
//! ```
//!
//! This **Linux**-specific behavior is controlled by the `unnamed-tmpfile` feature of this Crate,
//! which is enabled by default.
//!
//! ## Notes and Limitations
//!
//! * If the path of an [`AtomicWriteFile`] is a directory or a file that cannot be removed (due to
//!   permissions or special attributes), an error will be produced when the [`AtomicWriteFile`] is
//!   committed. This is in contrast with the standard `File`, which would instead produce an error
//!   at `open()` time.
//!
//! * [`AtomicWriteFile`] is designed so that the temporary files it creates are automatically
//!   removed if an error (such as a panic) occurs. However, if the process is interrupted abruptly
//!   (without unwinding or running destructors), temporary files may be left on the filesystem.
//!
//! * On Linux, with the `unnamed-tmpfile` feature (enabled by default), [`AtomicWriteFile`] uses
//!   unnamed temporary files. This ensures that, if the process is interrupted abruptly *before* a
//!   commit, the temporary file is automatically cleaned up by the operating system. However, if
//!   the process is interrupted *during* a commit, it's still possible (although unlikely) that a
//!   named temporary file will be left inside the destination directory.
//!
//! * On Linux, with the `unnamed-tmpfile` feature (enabled by default), [`AtomicWriteFile`]
//!   requires the `/proc` filesystem to be mounted. This makes [`AtomicWriteFile`] unsuitable for
//!   use in processes that run early at boot. Disable the `unnamed-tmpfile` feature if you need to
//!   run your program in situations where `/proc` is not available.
//!
//! * If the path of an [`AtomicWriteFile`] is a symlink to another file, the symlink is replaced,
//!   and the target of the original symlink is left untouched. If you intend to modify the file
//!   pointed by a symlink at open time, call [`Path::canonicalize()`] prior to calling
//!   [`AtomicWriteFile::open()`] or [`OpenOptions::open()`]. In the future, handling of symlinks
//!   will be better customizable.
//!
//! * Because [`AtomicWriteFile`] works by creating a temporary file, and then replacing the
//!   original file (see ["how it works"](#how-it-works) above), some metadata of the original file
//!   may be lost:
//!
//!   * On Unix, it is possible to preserve permissions and ownership of the original file.
//!     However, it is not generally possible to preserve the same owner user/group of the original
//!     file unless the process runs as root (or with the `CAP_CHOWN` capability on Linux). See
//!     [`OpenOptionsExt::try_preserve_owner()`](crate::unix::OpenOptionsExt::try_preserve_owner)
//!     for more details on the behavior of [`open()`](OpenOptions::open) when ownership cannot be
//!     preserved.
//!
//!   * On non-Unix platform, there is no support for preserving file permissions or ownership.
//!     Support may be added in the future.
//!
//!   * On all platforms, there is no support for preserving timestamps, ACLs (POSIX Access Control
//!     Lists), Linux extended attributes (xattrs), or SELinux contexts. Support may be added in
//!     the future.

#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(pointer_structural_match)]
#![warn(unreachable_pub)]
#![warn(unused_crate_dependencies)]
#![warn(unused_qualifications)]
#![doc(test(attr(deny(warnings))))]

use std::fmt::Arguments;
use std::fs::File;
use std::io::IoSlice;
use std::io::IoSliceMut;
use std::io::Read;
use std::io::Result;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::Path;

#[cfg(feature = "unstable-read_buf")]
use std::io::BorrowedCursor;

mod imp;

#[cfg(any(unix, target_os = "wasi"))]
mod fd;

#[cfg(unix)]
pub mod unix;

#[cfg(test)]
mod tests;

/// Options to configure how an [`AtomicWriteFile`] is opened.
///
/// This struct mimics the standard struct [`std::fs::OpenOptions`], and offers a subset of its
/// features that are applicable to [`AtomicWriteFile`].
///
/// Options can be set using methods like [`read()`](OpenOptions::read). Once the desired options
/// are set, the file can be opened using [`open()`](OpenOptions::open).
///
/// This crate offers some platform-specific extensions for `OpenOptions` in the form of traits:
///
/// * [`unix::OpenOptionsExt`]
///
/// # Notable differences between `std::fs::OpenOptions` and `atomic_write_file::OpenOptions`
///
/// The `OpenOptions` provided in this crate opens all files for writing by default, and the opened
/// file is always initially empty ("truncated"). As such, the following methods are not provided:
/// `write()`, `truncate()`, `append()`.
///
/// `create()` is not provided because a new file is always created if an original file does not
/// exist.
///
/// `create_new()` is also not provided because there is no way to ensure that a file never exists
/// from the time an [`AtomicWriteFile`] is opened to the time it is committed.
///
/// # Behavior when opening a file that already exists
///
/// When passing a path to [`open()`](OpenOptions::open) that points to a file that already exists,
/// [`AtomicWriteFile`] may preserve some of the metadata of the existing file (permissions,
/// ownership, and more). This behavior is platform-specific and can be controlled using the
/// platform-specific `OpenOptionsExt` traits. See also the ["notes and limitations" section on the
/// module-level documentations](crate#notes-and-limitations) for more information about what
/// metadata is preserved, what is not preserved, and in what circumstances.
///
/// # Examples
///
/// Opening a file for writing with default options (equivalent to a call to
/// [`AtomicWriteFile::open()`]):
///
/// ```
/// # let test_dir = option_env!("TEST_DIR").unwrap_or("target/test-files");
/// # std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
/// # std::env::set_current_dir(test_dir).expect("failed to move to test dir");
/// use atomic_write_file::OpenOptions;
/// let file = OpenOptions::new().open("foo.txt");
/// # std::mem::drop(file);
/// ```
///
/// Opening a file for both reading and writing:
///
/// ```
/// # let test_dir = option_env!("TEST_DIR").unwrap_or("target/test-files");
/// # std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
/// # std::env::set_current_dir(test_dir).expect("failed to move to test dir");
/// use atomic_write_file::OpenOptions;
/// let file = OpenOptions::new().read(true).open("foo.txt");
/// # std::mem::drop(file)
/// ```
#[derive(Clone, Debug)]
pub struct OpenOptions {
    inner: imp::OpenOptions,
}

impl OpenOptions {
    /// Create a set of options set to their default values.
    pub fn new() -> Self {
        Self {
            inner: imp::OpenOptions::new(),
        }
    }

    /// Sets the option for read access.
    ///
    /// If `true`, the file will be readable (other than being writeable) once opened using, for
    /// example, the [`Read`] trait. Note that if opening an already-existing file, the original
    /// file contents will not be readable. Only the new contents of the file will be readable.
    ///
    /// If `false` (the default), the file is opened in write-only mode.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// # let test_dir = option_env!("TEST_DIR").unwrap_or("target/test-files");
    /// # std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
    /// # std::env::set_current_dir(test_dir).expect("failed to move to test dir");
    /// use std::io::Seek;
    /// use std::io::Write;
    /// use std::io::read_to_string;
    /// use atomic_write_file::OpenOptions;
    ///
    /// let mut file = OpenOptions::new().read(true).open("foo.txt")?;
    /// writeln!(file, "hello")?;
    ///
    /// file.rewind()?;
    /// assert_eq!(read_to_string(&file)?, "hello\n");
    /// # Ok(())
    /// # }
    /// ```
    pub fn read(&mut self, read: bool) -> &mut Self {
        self.inner.read = read;
        self
    }

    /// Opens the file at `path` with this set of options.
    ///
    /// This has the same semantics as [`std::fs::OpenOptions::open()`], except that it returns an
    /// [`AtomicWriteFile`] instead of a [`File`].
    ///
    /// # Examples
    ///
    /// ```
    /// # let test_dir = option_env!("TEST_DIR").unwrap_or("target/test-files");
    /// # std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
    /// # std::env::set_current_dir(test_dir).expect("failed to move to test dir");
    /// use atomic_write_file::OpenOptions;
    /// let file = OpenOptions::new().read(true).open("foo.txt");
    /// # std::mem::drop(file)
    /// ```
    pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<AtomicWriteFile> {
        let path = path.as_ref().to_path_buf();
        let temporary_file = imp::TemporaryFile::open(&self.inner, &path)?;
        Ok(AtomicWriteFile {
            temporary_file,
            finalized: false,
        })
    }
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// A file whose contents become visible to users only after the file is committed.
///
/// An `AtomicWriteFile` is a file that is assigned to a path, but whose contents won't appear at
/// that path until the file is [committed](AtomicWriteFile::commit). If `AtomicWriteFile` is used
/// to open a file that already exists, the contents of the existing file will remain available
/// until the `AtomicWriteFile` is committed. During that time, the `AtomicWriteFile` may be used
/// to write new contents, but these new contents won't be visible until after the file is
/// committed.
///
/// Internally, `AtomicWriteFile` is implemented by initally opening a temporary file, and then
/// renaming the temporary file to its final path on commit. See the [module-level
/// documentation](crate) for more details about the implementation.
///
/// An `AtomicWriteFile` is automatically discarded when it goes out of scope (when it gets
/// dropped). Any error that occurs on drop is ignored. For this reason, if the file should not be
/// committed, it is highly recommended that `AtomicWriteFile` is discarded explicitly using the
/// [`discard()`](AtomicWriteFile::discard) method, which allows callers to detect errors on
/// cleanup. See [committing or discarding changes](#committing-or-discarding-changes) below for
/// more information.
///
/// # Opening an `AtomicWriteFile`
///
/// There are two ways to obtain an `AtomicWriteFile` struct:
///
/// * [`AtomicWriteFile::open()`]
/// * [`OpenOptions::open()`]
///
/// The first method opens a file at the specified path with some default options. The second
/// method using [`OpenOptions`] allows configuring how the file is opened.
///
/// # Compatibility with `std::fs::File`
///
/// `AtomicWriteFile` implements the same methods and traits of [`std::fs::File`], and aims to be
/// as much compatible with `File` as possible. In fact, `AtomicWriteFile` can be
/// [dereferenced](Deref) into a `File` struct: this means that you can use all methods provided by
/// [`File`] directly on an `AtomicWriteFile` (just like you can use all of [`str`] methods on a
/// [`String`]).
///
/// A reference to the wrapped `File` struct may also be explicitly obtained using
/// [`as_file()`](AtomicWriteFile::as_file) and [`as_file_mut()`](AtomicWriteFile::as_file_mut).
///
/// # Committing or discarding changes
///
/// `AtomicWriteFile` provides two additional methods that are not provided by [`File`]:
/// [`commit()`](AtomicWriteFile::commit) and [`discard()`](AtomicWriteFile::discard). These
/// methods can be called to save the new contents to the file path, or to destroy the new contents
/// and leave the original file (if any) unchaged, respectively.
///
/// Changes are automatically discarded also when `AtomicWriteFile` is dropped. Therefore calling
/// [`discard()`](AtomicWriteFile::discard) is not mandatory, but it is highly recommended because
/// the [`Drop`] implementation ignores all errors.
///
/// # Cloning
///
/// Cloning a `AtomicWriteFile` is not possible, because this would result in ambiguity and race
/// conditions when committing the file and its clones. It is however possible to clone the
/// underlaying [`File`] struct using [`try_clone()`](File::try_clone). Writes to this cloned
/// [`File`] however won't be atomic after the `AtomicWriteFile` is committed.
///
/// # Examples
///
/// Opening a file, writing new contents, and committing the changes:
///
/// ```
/// # fn main() -> std::io::Result<()> {
/// # let test_dir = option_env!("TEST_DIR").unwrap_or("target/test-files");
/// # std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
/// # std::env::set_current_dir(test_dir).expect("failed to move to test dir");
/// use std::io::Write;
/// use atomic_write_file::AtomicWriteFile;
///
/// let mut file = AtomicWriteFile::open("foo.txt")?; // if "foo.txt" already exists, it is not
///                                                   // initially truncated or deleted
/// writeln!(file, "hello")?; // "hello" is written to a temporary location; "foo.txt" (if it
///                           // exists) keeps its old contents after this write
///
/// file.commit()?; // only now "foo.txt" gets swapped with the new contents ("hello")
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct AtomicWriteFile {
    temporary_file: imp::TemporaryFile,
    finalized: bool,
}

impl AtomicWriteFile {
    /// Opens an atomically-written file at `path`.
    ///
    /// See [`OpenOptions`] for more details.
    ///
    /// # Examples
    ///
    /// ```
    /// # let test_dir = option_env!("TEST_DIR").unwrap_or("target/test-files");
    /// # std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
    /// # std::env::set_current_dir(test_dir).expect("failed to move to test dir");
    /// use atomic_write_file::AtomicWriteFile;
    /// let file = AtomicWriteFile::open("foo.txt");
    /// # std::mem::drop(file);
    /// ```
    #[inline]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<AtomicWriteFile> {
        OpenOptions::new().open(path)
    }

    /// Creates a new [`OpenOptions`] with default options.
    ///
    /// This is equivalent to [`OpenOptions::new()`], but allows for more readable code.
    ///
    /// # Examples
    ///
    /// ```
    /// # let test_dir = option_env!("TEST_DIR").unwrap_or("target/test-files");
    /// # std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
    /// # std::env::set_current_dir(test_dir).expect("failed to move to test dir");
    /// use atomic_write_file::AtomicWriteFile;
    /// let file = AtomicWriteFile::options().read(true).open("foo.txt");
    /// # std::mem::drop(file)
    /// ```
    #[inline]
    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    /// Returns a reference to the underlaying [`File`] struct.
    ///
    /// The returned reference can be used to inspect or manipulate the contents and metadata of
    /// this `AtomicWriteFile`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(any(unix, target_os = "wasi"))]
    /// # fn main() -> std::io::Result<()> {
    /// # let test_dir = option_env!("TEST_DIR").unwrap_or("target/test-files");
    /// # std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
    /// # std::env::set_current_dir(test_dir).expect("failed to move to test dir");
    /// use std::os::fd::AsRawFd;
    /// use atomic_write_file::AtomicWriteFile;
    ///
    /// let file = AtomicWriteFile::open("foo.txt")?;
    /// assert_eq!(file.as_raw_fd(), file.as_file().as_raw_fd());
    /// # Ok(())
    /// # }
    /// # #[cfg(not(any(unix, target_os = "wasi")))]
    /// # fn main() -> std::io::Result<()> {
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn as_file(&self) -> &File {
        &self.temporary_file.file
    }

    /// Returns a mutable reference to the underlaying [`File`] struct.
    ///
    /// The returned reference can be used to inspect or manipulate the contents and metadata of
    /// this `AtomicWriteFile`.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// # let test_dir = option_env!("TEST_DIR").unwrap_or("target/test-files");
    /// # std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
    /// # std::env::set_current_dir(test_dir).expect("failed to move to test dir");
    /// use std::io::Write;
    /// use atomic_write_file::AtomicWriteFile;
    ///
    /// let mut file = AtomicWriteFile::open("foo.txt")?;
    /// writeln!(file.as_file_mut(), "hello")?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn as_file_mut(&mut self) -> &mut File {
        &mut self.temporary_file.file
    }

    /// Saves the contents of this file to its path.
    ///
    /// After calling `commit()`, the `AtomicWriteFile` is consumed and can no longer be used.
    /// Clones of the underlaying [`File`] may still be used after calling `commit()`, although any
    /// write from that point onwards will no longer be atomic.
    ///
    /// See the documentation for [`AtomicWriteFile`] and the [module-level documentation](crate)
    /// for details about the internal implementation of `commit()`, as well as platform-specific
    /// details.
    ///
    /// This method is automatically called when `AtomicWriteFile` is dropped, although in that
    /// case any error produced by `commit()` is ignored.
    ///
    /// See also [`AtomicWriteFile::discard()`].
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// # let test_dir = option_env!("TEST_DIR").unwrap_or("target/test-files");
    /// # std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
    /// # std::env::set_current_dir(test_dir).expect("failed to move to test dir");
    /// use std::io::Write;
    /// use atomic_write_file::AtomicWriteFile;
    ///
    /// let file = AtomicWriteFile::open("foo.txt")?;
    /// writeln!(&file, "hello")?;
    /// file.commit()?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn commit(mut self) -> Result<()> {
        self._commit()
    }

    fn _commit(&mut self) -> Result<()> {
        if self.finalized {
            return Ok(());
        }
        self.finalized = true;
        self.sync_all()?;
        self.temporary_file.rename_file()
    }

    /// Discard the contents of this file, and leave its path unchanged.
    ///
    /// After calling `discard()`, the `AtomicWriteFile` is consumed and can no longer be used.
    /// Clones of the underlaying [`File`] may still be used after calling `discard()`.
    ///
    /// See also [`AtomicWriteFile::commit()`].
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// # let test_dir = option_env!("TEST_DIR").unwrap_or("target/test-files");
    /// # std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
    /// # std::env::set_current_dir(test_dir).expect("failed to move to test dir");
    /// use std::io::Write;
    /// use atomic_write_file::AtomicWriteFile;
    ///
    /// let file = AtomicWriteFile::open("foo.txt")?;
    /// writeln!(&file, "hello")?;
    /// file.discard()?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn discard(mut self) -> Result<()> {
        self._discard()
    }

    fn _discard(&mut self) -> Result<()> {
        if self.finalized {
            return Ok(());
        }
        self.finalized = true;
        self.temporary_file.remove_file()
    }
}

impl Drop for AtomicWriteFile {
    #[inline]
    fn drop(&mut self) {
        // Ignore all errors
        let _ = self._discard();
    }
}

impl Deref for AtomicWriteFile {
    type Target = File;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_file()
    }
}

impl DerefMut for AtomicWriteFile {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_file_mut()
    }
}

impl Read for AtomicWriteFile {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.temporary_file.file.read(buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<usize> {
        self.temporary_file.file.read_vectored(bufs)
    }

    #[inline]
    #[cfg(feature = "unstable-can_vector")]
    fn is_read_vectored(&self) -> bool {
        self.temporary_file.file.is_read_vectored()
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        self.temporary_file.file.read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        self.temporary_file.file.read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.temporary_file.file.read_exact(buf)
    }

    #[inline]
    #[cfg(feature = "unstable-read_buf")]
    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> Result<()> {
        self.temporary_file.file.read_buf(buf)
    }

    #[inline]
    #[cfg(feature = "unstable-read_buf")]
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> Result<()> {
        self.temporary_file.file.read_buf_exact(cursor)
    }
}

impl Read for &AtomicWriteFile {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        (&self.temporary_file.file).read(buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<usize> {
        (&self.temporary_file.file).read_vectored(bufs)
    }

    #[inline]
    #[cfg(feature = "unstable-can_vector")]
    fn is_read_vectored(&self) -> bool {
        self.temporary_file.file.is_read_vectored()
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        (&self.temporary_file.file).read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        (&self.temporary_file.file).read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        (&self.temporary_file.file).read_exact(buf)
    }

    #[inline]
    #[cfg(feature = "unstable-read_buf")]
    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> Result<()> {
        (&self.temporary_file.file).read_buf(buf)
    }

    #[inline]
    #[cfg(feature = "unstable-read_buf")]
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> Result<()> {
        (&self.temporary_file.file).read_buf_exact(cursor)
    }
}

impl Write for AtomicWriteFile {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.temporary_file.file.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> Result<()> {
        self.temporary_file.file.flush()
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<usize> {
        self.temporary_file.file.write_vectored(bufs)
    }

    #[inline]
    #[cfg(feature = "unstable-can_vector")]
    fn is_write_vectored(&self) -> bool {
        self.temporary_file.file.is_write_vectored()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.temporary_file.file.write_all(buf)
    }

    #[inline]
    #[cfg(feature = "unstable-can_vector")]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> Result<()> {
        self.temporary_file.file.write_all_vectored(bufs)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: Arguments<'_>) -> Result<()> {
        self.temporary_file.file.write_fmt(fmt)
    }
}

impl Write for &AtomicWriteFile {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        (&self.temporary_file.file).write(buf)
    }

    #[inline]
    fn flush(&mut self) -> Result<()> {
        (&self.temporary_file.file).flush()
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<usize> {
        (&self.temporary_file.file).write_vectored(bufs)
    }

    #[inline]
    #[cfg(feature = "unstable-can_vector")]
    fn is_write_vectored(&self) -> bool {
        (&self.temporary_file.file).is_write_vectored()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        (&self.temporary_file.file).write_all(buf)
    }

    #[inline]
    #[cfg(feature = "unstable-can_vector")]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> Result<()> {
        (&self.temporary_file.file).write_all_vectored(bufs)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: Arguments<'_>) -> Result<()> {
        (&self.temporary_file.file).write_fmt(fmt)
    }
}

impl Seek for AtomicWriteFile {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.temporary_file.file.seek(pos)
    }

    #[inline]
    fn rewind(&mut self) -> Result<()> {
        self.temporary_file.file.rewind()
    }

    #[inline]
    #[cfg(feature = "unstable-seek_stream_len")]
    fn stream_len(&mut self) -> Result<u64> {
        self.temporary_file.file.stream_len()
    }

    #[inline]
    fn stream_position(&mut self) -> Result<u64> {
        self.temporary_file.file.stream_position()
    }
}

impl Seek for &AtomicWriteFile {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        (&self.temporary_file.file).seek(pos)
    }

    #[inline]
    fn rewind(&mut self) -> Result<()> {
        (&self.temporary_file.file).rewind()
    }

    #[inline]
    #[cfg(feature = "unstable-seek_stream_len")]
    fn stream_len(&mut self) -> Result<u64> {
        (&self.temporary_file.file).stream_len()
    }

    #[inline]
    fn stream_position(&mut self) -> Result<u64> {
        (&self.temporary_file.file).stream_position()
    }
}
