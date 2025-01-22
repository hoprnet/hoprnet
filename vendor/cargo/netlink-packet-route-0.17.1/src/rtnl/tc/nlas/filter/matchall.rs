// SPDX-License-Identifier: MIT

/// Matchall filter
///
/// Matches all packets and performs an action on them.
use anyhow::Context;
use byteorder::{ByteOrder, NativeEndian};
use netlink_packet_utils::{
    nla::{self, DefaultNla, NlaBuffer, NlasIterator},
    parsers::parse_u32,
    traits::{Emitable, Parseable},
    DecodeError,
};

use crate::tc::{constants::*, Action};
pub const KIND: &str = "matchall";

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum Nla {
    Unspec(Vec<u8>),
    ClassId(u32),
    Act(Vec<Action>),
    Pcnt(Vec<u8>),
    Flags(u32),
    Other(DefaultNla),
}

impl nla::Nla for Nla {
    fn value_len(&self) -> usize {
        use self::Nla::*;
        match self {
            Unspec(b) | Pcnt(b) => b.len(),
            ClassId(_) | Flags(_) => 4,
            Act(acts) => acts.as_slice().buffer_len(),
            Other(attr) => attr.value_len(),
        }
    }

    fn emit_value(&self, buffer: &mut [u8]) {
        use self::Nla::*;
        match self {
            Unspec(b) | Pcnt(b) => buffer.copy_from_slice(b.as_slice()),
            ClassId(i) | Flags(i) => NativeEndian::write_u32(buffer, *i),
            Act(acts) => acts.as_slice().emit(buffer),
            Other(attr) => attr.emit_value(buffer),
        }
    }

    fn kind(&self) -> u16 {
        use self::Nla::*;
        match self {
            Unspec(_) => TCA_MATCHALL_UNSPEC,
            ClassId(_) => TCA_MATCHALL_CLASSID,
            Act(_) => TCA_MATCHALL_ACT,
            Pcnt(_) => TCA_MATCHALL_PCNT,
            Flags(_) => TCA_MATCHALL_FLAGS,
            Other(attr) => attr.kind(),
        }
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<NlaBuffer<&'a T>> for Nla {
    fn parse(buf: &NlaBuffer<&'a T>) -> Result<Self, DecodeError> {
        use self::Nla::*;
        let payload = buf.value();
        Ok(match buf.kind() {
            TCA_MATCHALL_UNSPEC => Unspec(payload.to_vec()),
            TCA_MATCHALL_CLASSID => ClassId(
                parse_u32(payload)
                    .context("failed to parse TCA_MATCHALL_UNSPEC")?,
            ),
            TCA_MATCHALL_ACT => {
                let mut acts = vec![];
                for act in NlasIterator::new(payload) {
                    let act = act.context("invalid TCA_MATCHALL_ACT")?;
                    acts.push(
                        Action::parse(&act)
                            .context("failed to parse TCA_MATCHALL_ACT")?,
                    );
                }
                Act(acts)
            }
            TCA_MATCHALL_PCNT => Pcnt(payload.to_vec()),
            TCA_MATCHALL_FLAGS => Flags(
                parse_u32(payload)
                    .context("failed to parse TCA_MATCHALL_FLAGS")?,
            ),
            _ => Other(
                DefaultNla::parse(buf).context("failed to parse u32 nla")?,
            ),
        })
    }
}
