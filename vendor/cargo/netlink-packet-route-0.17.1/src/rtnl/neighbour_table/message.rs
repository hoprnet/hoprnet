// SPDX-License-Identifier: MIT

use anyhow::Context;
use netlink_packet_utils::{
    traits::{Emitable, Parseable},
    DecodeError,
};

use crate::{
    nlas::neighbour_table::Nla, NeighbourTableHeader,
    NeighbourTableMessageBuffer,
};

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub struct NeighbourTableMessage {
    pub header: NeighbourTableHeader,
    pub nlas: Vec<Nla>,
}

impl Emitable for NeighbourTableMessage {
    fn buffer_len(&self) -> usize {
        self.header.buffer_len() + self.nlas.as_slice().buffer_len()
    }

    fn emit(&self, buffer: &mut [u8]) {
        self.header.emit(buffer);
        self.nlas.as_slice().emit(buffer);
    }
}

impl<'a, T: AsRef<[u8]> + 'a> Parseable<NeighbourTableMessageBuffer<&'a T>>
    for NeighbourTableMessage
{
    fn parse(
        buf: &NeighbourTableMessageBuffer<&'a T>,
    ) -> Result<Self, DecodeError> {
        Ok(NeighbourTableMessage {
            header: NeighbourTableHeader::parse(buf)
                .context("failed to parse neighbour table message header")?,
            nlas: Vec::<Nla>::parse(buf)
                .context("failed to parse neighbour table message NLAs")?,
        })
    }
}

impl<'a, T: AsRef<[u8]> + 'a> Parseable<NeighbourTableMessageBuffer<&'a T>>
    for Vec<Nla>
{
    fn parse(
        buf: &NeighbourTableMessageBuffer<&'a T>,
    ) -> Result<Self, DecodeError> {
        let mut nlas = vec![];
        for nla_buf in buf.nlas() {
            nlas.push(Nla::parse(&nla_buf?)?);
        }
        Ok(nlas)
    }
}
