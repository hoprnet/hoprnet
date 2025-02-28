use std::io;

pub(crate) fn timeout_err(msg: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, msg)
}
