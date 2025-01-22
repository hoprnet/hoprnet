// SPDX-License-Identifier: MIT

use netlink_packet_utils::{
    traits::{Emitable, Parseable},
    DecodeError,
};

use super::{NsidMessageBuffer, NSID_HEADER_LEN};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NsidHeader {
    pub rtgen_family: u8,
}

impl Emitable for NsidHeader {
    fn buffer_len(&self) -> usize {
        NSID_HEADER_LEN
    }

    fn emit(&self, buffer: &mut [u8]) {
        let mut packet = NsidMessageBuffer::new(buffer);
        packet.set_rtgen_family(self.rtgen_family);
    }
}

impl<T: AsRef<[u8]>> Parseable<NsidMessageBuffer<T>> for NsidHeader {
    fn parse(buf: &NsidMessageBuffer<T>) -> Result<Self, DecodeError> {
        Ok(NsidHeader {
            rtgen_family: buf.rtgen_family(),
        })
    }
}
