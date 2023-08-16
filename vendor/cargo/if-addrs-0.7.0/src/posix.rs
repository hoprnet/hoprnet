// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::sockaddr;
use libc::{freeifaddrs, getifaddrs, ifaddrs};
use std::net::IpAddr;
use std::{io, mem};

#[cfg(any(target_os = "linux", target_os = "android", target_os = "nacl"))]
pub fn do_broadcast(ifaddr: &ifaddrs) -> Option<IpAddr> {
    sockaddr::to_ipaddr(ifaddr.ifa_ifu)
}

#[cfg(any(
    target_os = "freebsd",
    target_os = "haiku",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "openbsd",
    target_os = "netbsd"
))]
pub fn do_broadcast(ifaddr: &ifaddrs) -> Option<IpAddr> {
    sockaddr::to_ipaddr(ifaddr.ifa_dstaddr)
}

pub struct IfAddrs {
    inner: *mut ifaddrs,
}

impl IfAddrs {
    #[allow(unsafe_code, clippy::new_ret_no_self)]
    pub fn new() -> io::Result<Self> {
        let mut ifaddrs = mem::MaybeUninit::uninit();

        unsafe {
            if -1 == getifaddrs(ifaddrs.as_mut_ptr()) {
                return Err(io::Error::last_os_error());
            }
            Ok(Self {
                inner: ifaddrs.assume_init(),
            })
        }
    }

    pub fn iter(&self) -> IfAddrsIterator {
        IfAddrsIterator { next: self.inner }
    }
}

impl Drop for IfAddrs {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        unsafe {
            freeifaddrs(self.inner);
        }
    }
}

pub struct IfAddrsIterator {
    next: *mut ifaddrs,
}

impl Iterator for IfAddrsIterator {
    type Item = ifaddrs;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        };

        Some(unsafe {
            let result = *self.next;
            self.next = (*self.next).ifa_next;

            result
        })
    }
}
