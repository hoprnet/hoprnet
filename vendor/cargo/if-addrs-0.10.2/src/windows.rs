// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use std::ffi::CStr;
use std::{io, ptr};
use windows_sys::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, ERROR_SUCCESS};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetAdaptersAddresses, GAA_FLAG_INCLUDE_PREFIX, GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER,
    GAA_FLAG_SKIP_FRIENDLY_NAME, GAA_FLAG_SKIP_MULTICAST, IP_ADAPTER_ADDRESSES_LH,
    IP_ADAPTER_PREFIX_XP, IP_ADAPTER_UNICAST_ADDRESS_LH,
};
use windows_sys::Win32::System::Memory::{
    GetProcessHeap, HeapAlloc, HeapFree, HEAP_NONE, HEAP_ZERO_MEMORY,
};

#[repr(transparent)]
pub struct IpAdapterAddresses(*const IP_ADAPTER_ADDRESSES_LH);

impl IpAdapterAddresses {
    #[allow(unsafe_code)]
    pub fn name(&self) -> String {
        unsafe { CStr::from_ptr((*self.0).AdapterName as _) }
            .to_string_lossy()
            .into_owned()
    }

    pub fn ipv4_index(&self) -> Option<u32> {
        let if_index = unsafe { (*self.0).Anonymous1.Anonymous.IfIndex };
        if if_index == 0 {
            None
        } else {
            Some(if_index)
        }
    }

    pub fn ipv6_index(&self) -> Option<u32> {
        let if_index = unsafe { (*self.0).Ipv6IfIndex };
        if if_index == 0 {
            None
        } else {
            Some(if_index)
        }
    }

    pub fn prefixes(&self) -> PrefixesIterator {
        PrefixesIterator {
            _head: unsafe { &*self.0 },
            next: unsafe { (*self.0).FirstPrefix },
        }
    }

    pub fn unicast_addresses(&self) -> UnicastAddressesIterator {
        UnicastAddressesIterator {
            _head: unsafe { &*self.0 },
            next: unsafe { (*self.0).FirstUnicastAddress },
        }
    }
}

pub struct IfAddrs {
    inner: IpAdapterAddresses,
}

impl IfAddrs {
    #[allow(unsafe_code)]
    pub fn new() -> io::Result<Self> {
        let mut buffersize = 15000;
        let mut ifaddrs: *mut IP_ADAPTER_ADDRESSES_LH;

        loop {
            unsafe {
                ifaddrs = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, buffersize as _)
                    as *mut IP_ADAPTER_ADDRESSES_LH;
                if ifaddrs.is_null() {
                    panic!("Failed to allocate buffer in get_if_addrs()");
                }

                let retcode = GetAdaptersAddresses(
                    0,
                    GAA_FLAG_SKIP_ANYCAST
                        | GAA_FLAG_SKIP_MULTICAST
                        | GAA_FLAG_SKIP_DNS_SERVER
                        | GAA_FLAG_INCLUDE_PREFIX
                        | GAA_FLAG_SKIP_FRIENDLY_NAME,
                    ptr::null_mut(),
                    ifaddrs,
                    &mut buffersize,
                );

                match retcode {
                    ERROR_SUCCESS => break,
                    ERROR_BUFFER_OVERFLOW => {
                        HeapFree(GetProcessHeap(), HEAP_NONE, ifaddrs as _);
                        buffersize *= 2;
                        continue;
                    }
                    _ => {
                        HeapFree(GetProcessHeap(), HEAP_NONE, ifaddrs as _);
                        return Err(io::Error::last_os_error());
                    }
                }
            }
        }

        Ok(Self {
            inner: IpAdapterAddresses(ifaddrs),
        })
    }

    pub fn iter(&self) -> IfAddrsIterator {
        IfAddrsIterator {
            _head: self,
            next: self.inner.0,
        }
    }
}

impl Drop for IfAddrs {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        unsafe {
            HeapFree(GetProcessHeap(), HEAP_NONE, self.inner.0 as _);
        }
    }
}

pub struct IfAddrsIterator<'a> {
    _head: &'a IfAddrs,
    next: *const IP_ADAPTER_ADDRESSES_LH,
}

impl<'a> Iterator for IfAddrsIterator<'a> {
    type Item = IpAdapterAddresses;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        };

        Some(unsafe {
            let result = &*self.next;
            self.next = (*self.next).Next;

            IpAdapterAddresses(result)
        })
    }
}

pub struct PrefixesIterator<'a> {
    _head: &'a IP_ADAPTER_ADDRESSES_LH,
    next: *const IP_ADAPTER_PREFIX_XP,
}

impl<'a> Iterator for PrefixesIterator<'a> {
    type Item = &'a IP_ADAPTER_PREFIX_XP;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        };

        Some(unsafe {
            let result = &*self.next;
            self.next = (*self.next).Next;

            result
        })
    }
}

pub struct UnicastAddressesIterator<'a> {
    _head: &'a IP_ADAPTER_ADDRESSES_LH,
    next: *const IP_ADAPTER_UNICAST_ADDRESS_LH,
}

impl<'a> Iterator for UnicastAddressesIterator<'a> {
    type Item = &'a IP_ADAPTER_UNICAST_ADDRESS_LH;

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        };

        Some(unsafe {
            let result = &*self.next;
            self.next = (*self.next).Next;

            result
        })
    }
}
