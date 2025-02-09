// SPDX-License-Identifier: MIT

#[macro_use]
extern crate bitflags;

pub mod rtnl;
pub use self::rtnl::*;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate netlink_packet_utils;
