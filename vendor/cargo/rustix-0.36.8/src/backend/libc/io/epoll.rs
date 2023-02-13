//! epoll support.
//!
//! This is an experiment, and it isn't yet clear whether epoll is the right
//! level of abstraction at which to introduce safety. But it works fairly well
//! in simple examples 🙂.
//!
//! # Examples
//!
//! ```rust,no_run
//! # #![cfg_attr(io_lifetimes_use_std, feature(io_safety))]
//! # #[cfg(feature = "net")]
//! # fn main() -> std::io::Result<()> {
//! use io_lifetimes::AsFd;
//! use rustix::io::epoll::{self, Epoll};
//! use rustix::io::{ioctl_fionbio, read, write};
//! use rustix::net::{
//!     accept, bind_v4, listen, socket, AddressFamily, Ipv4Addr, Protocol, SocketAddrV4,
//!     SocketType,
//! };
//! use std::os::unix::io::AsRawFd;
//!
//! // Create a socket and listen on it.
//! let listen_sock = socket(AddressFamily::INET, SocketType::STREAM, Protocol::default())?;
//! bind_v4(&listen_sock, &SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))?;
//! listen(&listen_sock, 1)?;
//!
//! // Create an epoll object. Using `Owning` here means the epoll object will
//! // take ownership of the file descriptors registered with it.
//! let epoll = Epoll::new(epoll::CreateFlags::CLOEXEC, epoll::Owning::new())?;
//!
//! // Remember the socket raw fd, which we use for comparisons only.
//! let raw_listen_sock = listen_sock.as_fd().as_raw_fd();
//!
//! // Register the socket with the epoll object.
//! epoll.add(listen_sock, epoll::EventFlags::IN)?;
//!
//! // Process events.
//! let mut event_list = epoll::EventVec::with_capacity(4);
//! loop {
//!     epoll.wait(&mut event_list, -1)?;
//!     for (_event_flags, target) in &event_list {
//!         if target.as_raw_fd() == raw_listen_sock {
//!             // Accept a new connection, set it to non-blocking, and
//!             // register to be notified when it's ready to write to.
//!             let conn_sock = accept(&*target)?;
//!             ioctl_fionbio(&conn_sock, true)?;
//!             epoll.add(conn_sock, epoll::EventFlags::OUT | epoll::EventFlags::ET)?;
//!         } else {
//!             // Write a message to the stream and then unregister it.
//!             write(&*target, b"hello\n")?;
//!             let _ = epoll.del(target)?;
//!         }
//!     }
//! }
//! # }
//! # #[cfg(not(feature = "net"))]
//! # fn main() {}
//! ```

use super::super::c;
use super::super::conv::{ret, ret_owned_fd, ret_u32};
use crate::fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd, RawFd};
#[cfg(not(feature = "rustc-dep-of-std"))]
use crate::fd::{FromRawFd, IntoRawFd};
use crate::io;
use alloc::vec::Vec;
use bitflags::bitflags;
use core::convert::TryInto;
use core::marker::PhantomData;
use core::ptr::{null, null_mut};

#[doc(inline)]
pub use crate::io::context::*;

bitflags! {
    /// `EPOLL_*` for use with [`Epoll::new`].
    pub struct CreateFlags: c::c_int {
        /// `EPOLL_CLOEXEC`
        const CLOEXEC = c::EPOLL_CLOEXEC;
    }
}

bitflags! {
    /// `EPOLL*` for use with [`Epoll::add`].
    #[derive(Default)]
    pub struct EventFlags: u32 {
        /// `EPOLLIN`
        const IN = c::EPOLLIN as u32;

        /// `EPOLLOUT`
        const OUT = c::EPOLLOUT as u32;

        /// `EPOLLPRI`
        const PRI = c::EPOLLPRI as u32;

        /// `EPOLLERR`
        const ERR = c::EPOLLERR as u32;

        /// `EPOLLHUP`
        const HUP = c::EPOLLHUP as u32;

        /// `EPOLLET`
        const ET = c::EPOLLET as u32;

        /// `EPOLLONESHOT`
        const ONESHOT = c::EPOLLONESHOT as u32;

        /// `EPOLLWAKEUP`
        const WAKEUP = c::EPOLLWAKEUP as u32;

        /// `EPOLLEXCLUSIVE`
        #[cfg(not(target_os = "android"))]
        const EXCLUSIVE = c::EPOLLEXCLUSIVE as u32;
    }
}

/// An "epoll", an interface to an OS object allowing one to repeatedly wait
/// for events from a set of file descriptors efficiently.
pub struct Epoll<Context: self::Context> {
    epoll_fd: OwnedFd,
    context: Context,
}

impl<Context: self::Context> Epoll<Context> {
    /// `epoll_create1(flags)`—Creates a new `Epoll`.
    ///
    /// Use the [`CreateFlags::CLOEXEC`] flag to prevent the resulting file
    /// descriptor from being implicitly passed across `exec` boundaries.
    #[inline]
    #[doc(alias = "epoll_create1")]
    pub fn new(flags: CreateFlags, context: Context) -> io::Result<Self> {
        // Safety: We're calling `epoll_create1` via FFI and we know how it
        // behaves.
        unsafe {
            Ok(Self {
                epoll_fd: ret_owned_fd(c::epoll_create1(flags.bits()))?,
                context,
            })
        }
    }

    /// `epoll_ctl(self, EPOLL_CTL_ADD, data, event)`—Adds an element to an
    /// `Epoll`.
    ///
    /// This registers interest in any of the events set in `events` occurring
    /// on the file descriptor associated with `data`.
    #[doc(alias = "epoll_ctl")]
    pub fn add(
        &self,
        data: Context::Data,
        event_flags: EventFlags,
    ) -> io::Result<Ref<'_, Context::Target>> {
        // Safety: We're calling `epoll_ctl` via FFI and we know how it
        // behaves.
        unsafe {
            let target = self.context.acquire(data);
            let raw_fd = target.as_fd().as_raw_fd();
            let encoded = self.context.encode(target);
            ret(c::epoll_ctl(
                self.epoll_fd.as_fd().as_raw_fd(),
                c::EPOLL_CTL_ADD,
                raw_fd,
                &mut c::epoll_event {
                    events: event_flags.bits(),
                    r#u64: encoded,
                },
            ))?;
            Ok(self.context.decode(encoded))
        }
    }

    /// `epoll_ctl(self, EPOLL_CTL_MOD, target, event)`—Modifies an element in
    /// this `Epoll`.
    ///
    /// This sets the events of interest with `target` to `events`.
    #[doc(alias = "epoll_ctl")]
    pub fn mod_(
        &self,
        target: Ref<'_, Context::Target>,
        event_flags: EventFlags,
    ) -> io::Result<()> {
        let raw_fd = target.as_fd().as_raw_fd();
        let encoded = self.context.encode(target);
        // Safety: We're calling `epoll_ctl` via FFI and we know how it
        // behaves.
        unsafe {
            ret(c::epoll_ctl(
                self.epoll_fd.as_fd().as_raw_fd(),
                c::EPOLL_CTL_MOD,
                raw_fd,
                &mut c::epoll_event {
                    events: event_flags.bits(),
                    r#u64: encoded,
                },
            ))
        }
    }

    /// `epoll_ctl(self, EPOLL_CTL_DEL, target, NULL)`—Removes an element in
    /// this `Epoll`.
    ///
    /// This also returns the owning `Data`.
    #[doc(alias = "epoll_ctl")]
    pub fn del(&self, target: Ref<'_, Context::Target>) -> io::Result<Context::Data> {
        // Safety: We're calling `epoll_ctl` via FFI and we know how it
        // behaves.
        unsafe {
            let raw_fd = target.as_fd().as_raw_fd();
            ret(c::epoll_ctl(
                self.epoll_fd.as_fd().as_raw_fd(),
                c::EPOLL_CTL_DEL,
                raw_fd,
                null_mut(),
            ))?;
        }
        Ok(self.context.release(target))
    }

    /// `epoll_wait(self, events, timeout)`—Waits for registered events of
    /// interest.
    ///
    /// For each event of interest, an element is written to `events`. On
    /// success, this returns the number of written elements.
    #[doc(alias = "epoll_wait")]
    pub fn wait<'context>(
        &'context self,
        event_list: &mut EventVec<'context, Context>,
        timeout: c::c_int,
    ) -> io::Result<()> {
        // Safety: We're calling `epoll_wait` via FFI and we know how it
        // behaves.
        unsafe {
            event_list.events.set_len(0);
            let nfds = ret_u32(c::epoll_wait(
                self.epoll_fd.as_fd().as_raw_fd(),
                event_list.events.as_mut_ptr().cast::<c::epoll_event>(),
                event_list.events.capacity().try_into().unwrap_or(i32::MAX),
                timeout,
            ))?;
            event_list.events.set_len(nfds as usize);
            event_list.context = &self.context;
        }

        Ok(())
    }
}

#[cfg(not(feature = "rustc-dep-of-std"))]
impl<'context, T: AsFd + Into<OwnedFd> + From<OwnedFd>> AsRawFd for Epoll<Owning<'context, T>> {
    fn as_raw_fd(&self) -> RawFd {
        self.epoll_fd.as_raw_fd()
    }
}

#[cfg(not(feature = "rustc-dep-of-std"))]
impl<'context, T: AsFd + Into<OwnedFd> + From<OwnedFd>> IntoRawFd for Epoll<Owning<'context, T>> {
    fn into_raw_fd(self) -> RawFd {
        self.epoll_fd.into_raw_fd()
    }
}

#[cfg(not(feature = "rustc-dep-of-std"))]
impl<'context, T: AsFd + Into<OwnedFd> + From<OwnedFd>> FromRawFd for Epoll<Owning<'context, T>> {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self {
            epoll_fd: OwnedFd::from_raw_fd(fd),
            context: Owning::new(),
        }
    }
}

#[cfg(not(feature = "rustc-dep-of-std"))]
impl<'context, T: AsFd + Into<OwnedFd> + From<OwnedFd>> AsFd for Epoll<Owning<'context, T>> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.epoll_fd.as_fd()
    }
}

#[cfg(not(feature = "rustc-dep-of-std"))]
impl<'context, T: AsFd + Into<OwnedFd> + From<OwnedFd>> From<Epoll<Owning<'context, T>>>
    for OwnedFd
{
    fn from(epoll: Epoll<Owning<'context, T>>) -> Self {
        epoll.epoll_fd
    }
}

#[cfg(not(feature = "rustc-dep-of-std"))]
impl<'context, T: AsFd + Into<OwnedFd> + From<OwnedFd>> From<OwnedFd>
    for Epoll<Owning<'context, T>>
{
    fn from(fd: OwnedFd) -> Self {
        Self {
            epoll_fd: fd,
            context: Owning::new(),
        }
    }
}

/// An iterator over the `Event`s in an `EventVec`.
pub struct Iter<'context, Context: self::Context> {
    iter: core::slice::Iter<'context, Event>,
    context: *const Context,
    _phantom: PhantomData<&'context Context>,
}

impl<'context, Context: self::Context> Iterator for Iter<'context, Context> {
    type Item = (EventFlags, Ref<'context, Context::Target>);

    fn next(&mut self) -> Option<Self::Item> {
        // Safety: `self.context` is guaranteed to be valid because we hold
        // `'context` for it. And we know this event is associated with this
        // context because `wait` sets both.
        self.iter.next().map(|event| {
            (event.event_flags, unsafe {
                (*self.context).decode(event.encoded)
            })
        })
    }
}

/// A record of an event that occurred.
#[repr(C)]
#[cfg_attr(
    any(
        all(
            target_arch = "x86",
            not(target_env = "musl"),
            not(target_os = "android"),
        ),
        target_arch = "x86_64",
    ),
    repr(packed)
)]
struct Event {
    // Match the layout of `c::epoll_event`. We just use a `u64` instead of
    // the full union; `Context` implementations will simply need to deal with
    // casting the value into and out of the `u64` themselves.
    event_flags: EventFlags,
    encoded: u64,
}

/// A vector of `Event`s, plus context for interpreting them.
pub struct EventVec<'context, Context: self::Context> {
    events: Vec<Event>,
    context: *const Context,
    _phantom: PhantomData<&'context Context>,
}

impl<'context, Context: self::Context> EventVec<'context, Context> {
    /// Constructs an `EventVec` with memory for `capacity` `Event`s.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
            context: null(),
            _phantom: PhantomData,
        }
    }

    /// Returns the current `Event` capacity of this `EventVec`.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.events.capacity()
    }

    /// Reserves enough memory for at least `additional` more `Event`s.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.events.reserve(additional);
    }

    /// Reserves enough memory for exactly `additional` more `Event`s.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.events.reserve_exact(additional);
    }

    /// Clears all the `Events` out of this `EventVec`.
    #[inline]
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Shrinks the capacity of this `EventVec` as much as possible.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.events.shrink_to_fit();
    }

    /// Returns an iterator over the `Event`s in this `EventVec`.
    #[inline]
    pub fn iter(&self) -> Iter<'_, Context> {
        Iter {
            iter: self.events.iter(),
            context: self.context,
            _phantom: PhantomData,
        }
    }

    /// Returns the number of `Event`s logically contained in this `EventVec`.
    #[inline]
    pub fn len(&mut self) -> usize {
        self.events.len()
    }

    /// Tests whether this `EventVec` is logically empty.
    #[inline]
    pub fn is_empty(&mut self) -> bool {
        self.events.is_empty()
    }
}

impl<'context, Context: self::Context> IntoIterator for &'context EventVec<'context, Context> {
    type IntoIter = Iter<'context, Context>;
    type Item = (EventFlags, Ref<'context, Context::Target>);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
