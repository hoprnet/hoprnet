// SPDX-License-Identifier: MIT

/// Nat action
///
/// The nat action maps one IP prefix to another
use std::net::Ipv4Addr;

use netlink_packet_utils::{
    nla::{self, DefaultNla, NlaBuffer},
    traits::{Emitable, Parseable},
    DecodeError,
};

use crate::tc::{constants::*, TC_GEN_BUF_LEN};

pub const KIND: &str = "nat";
pub const TC_NAT_BUF_LEN: usize = TC_GEN_BUF_LEN + 16;

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum Nla {
    Unspec(Vec<u8>),
    Tm(Vec<u8>),
    Parms(TcNat),
    Other(DefaultNla),
}

impl nla::Nla for Nla {
    fn value_len(&self) -> usize {
        use self::Nla::*;
        match self {
            Unspec(bytes) | Tm(bytes) => bytes.len(),
            Parms(_) => TC_NAT_BUF_LEN,
            Other(attr) => attr.value_len(),
        }
    }

    fn emit_value(&self, buffer: &mut [u8]) {
        use self::Nla::*;
        match self {
            Unspec(bytes) | Tm(bytes) => {
                buffer.copy_from_slice(bytes.as_slice())
            }
            Parms(p) => p.emit(buffer),
            Other(attr) => attr.emit_value(buffer),
        }
    }
    fn kind(&self) -> u16 {
        use self::Nla::*;
        match self {
            Unspec(_) => TCA_NAT_UNSPEC,
            Tm(_) => TCA_NAT_TM,
            Parms(_) => TCA_NAT_PARMS,
            Other(nla) => nla.kind(),
        }
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<NlaBuffer<&'a T>> for Nla {
    fn parse(buf: &NlaBuffer<&'a T>) -> Result<Self, DecodeError> {
        use self::Nla::*;
        let payload = buf.value();
        Ok(match buf.kind() {
            TCA_NAT_UNSPEC => Unspec(payload.to_vec()),
            TCA_NAT_TM => Tm(payload.to_vec()),
            TCA_NAT_PARMS => {
                Parms(TcNat::parse(&TcNatBuffer::new_checked(payload)?)?)
            }
            _ => Other(DefaultNla::parse(buf)?),
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct TcNat {
    pub index: u32,
    pub capab: u32,
    pub action: i32,
    pub refcnt: i32,
    pub bindcnt: i32,

    pub old_addr: Vec<u8>,
    pub new_addr: Vec<u8>,
    pub mask: Vec<u8>,
    pub flags: u32,
}

buffer!(TcNatBuffer(TC_NAT_BUF_LEN) {
    index: (u32, 0..4),
    capab: (u32, 4..8),
    action: (i32, 8..12),
    refcnt: (i32, 12..16),
    bindcnt: (i32, 16..20),

    old_addr: (slice, TC_GEN_BUF_LEN..(TC_GEN_BUF_LEN+4)),
    new_addr: (slice, (TC_GEN_BUF_LEN +4)..(TC_GEN_BUF_LEN+8)),
    mask: (slice, (TC_GEN_BUF_LEN +8)..(TC_GEN_BUF_LEN+12)),
    flags: (u32, (TC_GEN_BUF_LEN+12)..TC_NAT_BUF_LEN),
});

impl TcNat {
    pub fn set_new_addr(mut self, target: Ipv4Addr) -> Self {
        self.new_addr = target.octets().to_vec();
        self
    }

    pub fn set_old_addr(mut self, target: Ipv4Addr) -> Self {
        self.old_addr = target.octets().to_vec();
        self
    }

    pub fn set_prefix(mut self, prefix_len: usize) -> Self {
        assert!(prefix_len <= 32);

        let prefix: u32 = if prefix_len == 0 {
            0x0
        } else {
            !((1 << (32 - prefix_len)) - 1)
        };
        self.mask = prefix.to_be_bytes().to_vec();

        self
    }

    pub fn egress(mut self) -> Self {
        self.flags = TCA_NAT_FLAG_EGRESS;
        self
    }
}
impl Emitable for TcNat {
    fn buffer_len(&self) -> usize {
        TC_NAT_BUF_LEN
    }

    fn emit(&self, buffer: &mut [u8]) {
        let mut packet = TcNatBuffer::new(buffer);
        packet.set_index(self.index);
        packet.set_capab(self.capab);
        packet.set_action(self.action);
        packet.set_refcnt(self.refcnt);
        packet.set_bindcnt(self.bindcnt);

        packet.old_addr_mut().copy_from_slice(&self.old_addr[0..4]);
        packet.new_addr_mut().copy_from_slice(&self.new_addr[0..4]);
        packet.mask_mut().copy_from_slice(&self.mask[0..4]);
        packet.set_flags(self.flags);
    }
}

impl<'buf, T: AsRef<[u8]> + ?Sized> Parseable<TcNatBuffer<&'buf T>> for TcNat {
    fn parse(buf: &TcNatBuffer<&'buf T>) -> Result<Self, DecodeError> {
        Ok(Self {
            index: buf.index(),
            capab: buf.capab(),
            action: buf.action(),
            refcnt: buf.refcnt(),
            bindcnt: buf.bindcnt(),
            old_addr: buf.old_addr().to_vec(),
            new_addr: buf.new_addr().to_vec(),
            mask: buf.mask().to_vec(),
            flags: buf.flags(),
        })
    }
}
