use crate::AtomicWriteFile;
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;
use std::os::fd::BorrowedFd;
use std::os::fd::RawFd;

impl AsFd for AtomicWriteFile {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.temporary_file.file.as_fd()
    }
}

impl AsRawFd for AtomicWriteFile {
    fn as_raw_fd(&self) -> RawFd {
        self.temporary_file.file.as_raw_fd()
    }
}
