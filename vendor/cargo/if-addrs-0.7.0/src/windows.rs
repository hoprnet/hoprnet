// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use libc::{self, c_char, c_int, c_ulong, c_void, size_t};
use std::ffi::CStr;
use std::{io, ptr};
use winapi::shared::minwindef::DWORD;
use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::shared::ws2def::SOCKADDR;

#[repr(C)]
pub struct SocketAddress {
    pub lp_socket_address: *const SOCKADDR,
    pub i_socket_address_length: c_int,
}
#[repr(C)]
pub struct IpAdapterUnicastAddress {
    pub length: c_ulong,
    pub flags: DWORD,
    pub next: *const IpAdapterUnicastAddress,
    // Loads more follows, but I'm not bothering to map these for now
    pub address: SocketAddress,
}
#[repr(C)]
pub struct IpAdapterPrefix {
    pub length: c_ulong,
    pub flags: DWORD,
    pub next: *const IpAdapterPrefix,
    pub address: SocketAddress,
    pub prefix_length: c_ulong,
}
#[repr(C)]
pub struct IpAdapterAddresses {
    pub length: c_ulong,
    pub if_index: DWORD,
    next: *const IpAdapterAddresses,
    adapter_name: *const c_char,
    first_unicast_address: *const IpAdapterUnicastAddress,
    first_anycast_address: *const c_void,
    first_multicast_address: *const c_void,
    first_dns_server_address: *const c_void,
    dns_suffix: *const c_void,
    description: *const c_void,
    friendly_name: *const c_void,
    physical_address: [c_char; 8],
    physical_address_length: DWORD,
    flags: DWORD,
    mtu: DWORD,
    if_type: DWORD,
    oper_status: c_int,
    ipv6_if_index: DWORD,
    zone_indices: [DWORD; 16],
    // Loads more follows, but I'm not bothering to map these for now
    first_prefix: *const IpAdapterPrefix,
}

impl IpAdapterAddresses {
    #[allow(unsafe_code)]
    pub fn name(&self) -> String {
        unsafe { CStr::from_ptr(self.adapter_name) }
            .to_string_lossy()
            .into_owned()
    }

    pub fn prefixes(&self) -> PrefixesIterator {
        PrefixesIterator {
            _head: self,
            next: self.first_prefix,
        }
    }

    pub fn unicast_addresses(&self) -> UnicastAddressesIterator {
        UnicastAddressesIterator {
            _head: self,
            next: self.first_unicast_address,
        }
    }
}

#[link(name = "iphlpapi")]
extern "system" {
    /// Get adapter's addresses.
    fn GetAdaptersAddresses(
        family: c_ulong,
        flags: c_ulong,
        reserved: *const c_void,
        addresses: *const IpAdapterAddresses,
        size: *mut c_ulong,
    ) -> c_ulong;
}

pub struct IfAddrs {
    inner: *const IpAdapterAddresses,
}

impl IfAddrs {
    #[allow(unsafe_code)]
    pub fn new() -> io::Result<Self> {
        let mut buffersize: c_ulong = 15000;
        let mut ifaddrs: *const IpAdapterAddresses;

        loop {
            unsafe {
                ifaddrs = libc::malloc(buffersize as size_t) as *mut IpAdapterAddresses;
                if ifaddrs.is_null() {
                    panic!("Failed to allocate buffer in get_if_addrs()");
                }

                let retcode = GetAdaptersAddresses(
                    0,
                    // GAA_FLAG_SKIP_ANYCAST       |
                    // GAA_FLAG_SKIP_MULTICAST     |
                    // GAA_FLAG_SKIP_DNS_SERVER    |
                    // GAA_FLAG_INCLUDE_PREFIX     |
                    // GAA_FLAG_SKIP_FRIENDLY_NAME
                    0x3e,
                    ptr::null(),
                    ifaddrs,
                    &mut buffersize,
                );

                match retcode {
                    ERROR_SUCCESS => break,
                    111 => {
                        libc::free(ifaddrs as *mut c_void);
                        buffersize *= 2;
                        continue;
                    }
                    _ => return Err(io::Error::last_os_error()),
                }
            }
        }

        Ok(Self { inner: ifaddrs })
    }

    pub fn iter(&self) -> IfAddrsIterator {
        IfAddrsIterator {
            _head: self,
            next: self.inner,
        }
    }
}

impl Drop for IfAddrs {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        unsafe {
            libc::free(self.inner as *mut c_void);
        }
    }
}

pub struct IfAddrsIterator<'a> {
    _head: &'a IfAddrs,
    next: *const IpAdapterAddresses,
}

impl<'a> Iterator for IfAddrsIterator<'a> {
    type Item = &'a IpAdapterAddresses;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        };

        Some(unsafe {
            let result = &*self.next;
            self.next = (*self.next).next;

            result
        })
    }
}

pub struct PrefixesIterator<'a> {
    _head: &'a IpAdapterAddresses,
    next: *const IpAdapterPrefix,
}

impl<'a> Iterator for PrefixesIterator<'a> {
    type Item = &'a IpAdapterPrefix;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        };

        Some(unsafe {
            let result = &*self.next;
            self.next = (*self.next).next;

            result
        })
    }
}

pub struct UnicastAddressesIterator<'a> {
    _head: &'a IpAdapterAddresses,
    next: *const IpAdapterUnicastAddress,
}

impl<'a> Iterator for UnicastAddressesIterator<'a> {
    type Item = &'a IpAdapterUnicastAddress;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        };

        Some(unsafe {
            let result = &*self.next;
            self.next = (*self.next).next;

            result
        })
    }
}
