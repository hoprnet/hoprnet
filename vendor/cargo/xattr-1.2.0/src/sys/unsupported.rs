use std::ffi::{OsStr, OsString};
use std::io;
use std::os::unix::io::BorrowedFd;
use std::path::Path;

use crate::UnsupportedPlatformError;

pub const ENOATTR: i32 = 0;

/// An iterator over a set of extended attributes names.
#[derive(Clone, Debug)]
pub struct XAttrs;

impl Iterator for XAttrs {
    type Item = OsString;
    fn next(&mut self) -> Option<OsString> {
        unreachable!("this should never exist")
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        unreachable!("this should never exist")
    }
}

pub fn get_fd(_: BorrowedFd<'_>, _: &OsStr) -> io::Result<Vec<u8>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        UnsupportedPlatformError,
    ))
}

pub fn set_fd(_: BorrowedFd<'_>, _: &OsStr, _: &[u8]) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        UnsupportedPlatformError,
    ))
}

pub fn remove_fd(_: BorrowedFd<'_>, _: &OsStr) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        UnsupportedPlatformError,
    ))
}

pub fn list_fd(_: BorrowedFd<'_>) -> io::Result<XAttrs> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        UnsupportedPlatformError,
    ))
}

pub fn get_path(_: &Path, _: &OsStr, _: bool) -> io::Result<Vec<u8>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        UnsupportedPlatformError,
    ))
}

pub fn set_path(_: &Path, _: &OsStr, _: &[u8], _: bool) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        UnsupportedPlatformError,
    ))
}

pub fn remove_path(_: &Path, _: &OsStr, _: bool) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        UnsupportedPlatformError,
    ))
}

pub fn list_path(_: &Path, _: bool) -> io::Result<XAttrs> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        UnsupportedPlatformError,
    ))
}
