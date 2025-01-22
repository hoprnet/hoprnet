// SPDX-License-Identifier: MIT

use futures::stream::StreamExt;
use netlink_packet_core::{NetlinkMessage, NLM_F_ACK, NLM_F_REQUEST};
use netlink_packet_route::{
    link::nlas::{Info, InfoBondPort, InfoPortData, InfoPortKind, Nla},
    LinkMessage, RtnlMessage,
};

use crate::{try_nl, Error, Handle};

pub struct BondPortSetRequest {
    handle: Handle,
    index: u32,
    port_nlas: Vec<InfoBondPort>,
}

impl BondPortSetRequest {
    pub(crate) fn new(handle: Handle, index: u32) -> Self {
        BondPortSetRequest {
            handle,
            index,
            port_nlas: Vec::new(),
        }
    }

    /// Execute the request.
    pub async fn execute(self) -> Result<(), Error> {
        let BondPortSetRequest {
            mut handle,
            index,
            port_nlas,
        } = self;

        let mut message = LinkMessage::default();
        message.header.index = index;
        message.nlas.push(Nla::Info(vec![
            Info::PortKind(InfoPortKind::Bond),
            Info::PortData(InfoPortData::BondPort(port_nlas)),
        ]));

        let mut req = NetlinkMessage::from(RtnlMessage::NewLink(message));
        req.header.flags = NLM_F_REQUEST | NLM_F_ACK;

        let mut response = handle.request(req)?;
        while let Some(message) = response.next().await {
            try_nl!(message);
        }
        Ok(())
    }

    /// Return a mutable reference to the Vec<InfoBondPort>
    pub fn info_port_nlas_mut(&mut self) -> &mut Vec<InfoBondPort> {
        &mut self.port_nlas
    }

    /// Adds the `queue_id` attribute to the bond port
    /// This is equivalent to
    /// `ip link set name NAME type bond_slave queue_id QUEUE_ID`.
    pub fn queue_id(mut self, queue_id: u16) -> Self {
        self.port_nlas.push(InfoBondPort::QueueId(queue_id));
        self
    }

    /// Adds the `prio` attribute to the bond port
    /// This is equivalent to `ip link set name NAME type bond_slave prio PRIO`.
    pub fn prio(mut self, prio: i32) -> Self {
        self.port_nlas.push(InfoBondPort::Prio(prio));
        self
    }
}
