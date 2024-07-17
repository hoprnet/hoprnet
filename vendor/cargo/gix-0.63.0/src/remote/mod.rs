use std::borrow::Cow;

use crate::bstr::BStr;

/// The direction of an operation carried out (or to be carried out) through a remote.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum Direction {
    /// Push local changes to the remote.
    Push,
    /// Fetch changes from the remote to the local repository.
    Fetch,
}

impl Direction {
    /// Return ourselves as string suitable for use as verb in an english sentence.
    pub fn as_str(&self) -> &'static str {
        match self {
            Direction::Push => "push",
            Direction::Fetch => "fetch",
        }
    }
}

/// The name of a remote, either interpreted as symbol like `origin` or as url as returned by [`Remote::name()`][crate::Remote::name()].
#[derive(Debug, PartialEq, Eq, Clone, Ord, PartialOrd, Hash)]
pub enum Name<'repo> {
    /// A symbolic name, like `origin`.
    /// Note that it has not necessarily been validated yet.
    Symbol(Cow<'repo, str>),
    /// A url pointing to the remote host directly.
    Url(Cow<'repo, BStr>),
}

///
#[allow(clippy::empty_docs)]
pub mod name;

mod build;

mod errors;
pub use errors::find;

///
#[allow(clippy::empty_docs)]
pub mod init;

///
#[allow(clippy::empty_docs)]
pub mod fetch;

///
#[cfg(any(feature = "async-network-client", feature = "blocking-network-client"))]
pub mod connect;

#[cfg(any(feature = "async-network-client", feature = "blocking-network-client"))]
mod connection;
#[cfg(any(feature = "async-network-client", feature = "blocking-network-client"))]
pub use connection::{ref_map, AuthenticateFn, Connection};

///
#[allow(clippy::empty_docs)]
pub mod save;

mod access;
///
#[allow(clippy::empty_docs)]
pub mod url;
