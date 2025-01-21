// SPDX-License-Identifier: MIT

use netlink_packet_core::{
    NetlinkHeader, NetlinkMessage, NetlinkPayload, NLM_F_ACK, NLM_F_CREATE,
    NLM_F_EXCL, NLM_F_REQUEST,
};
use netlink_packet_route::{
    constants::{AF_INET, FR_ACT_TO_TBL, RT_TABLE_DEFAULT},
    rule, RtnlMessage, RuleHeader, RuleMessage,
};
use netlink_sys::{protocols::NETLINK_ROUTE, Socket, SocketAddr};

fn main() {
    let mut socket = Socket::new(NETLINK_ROUTE).unwrap();
    let _port_number = socket.bind_auto().unwrap().port_number();
    socket.connect(&SocketAddr::new(0, 0)).unwrap();

    let mut rule_msg_hdr = RuleHeader::default();
    rule_msg_hdr.family = AF_INET as u8;
    rule_msg_hdr.table = RT_TABLE_DEFAULT;
    rule_msg_hdr.action = FR_ACT_TO_TBL;

    let mut rule_msg = RuleMessage::default();
    rule_msg.header = rule_msg_hdr;
    rule_msg.nlas = vec![
        rule::Nla::Table(254),
        rule::Nla::SuppressPrefixLen(4294967295),
        rule::Nla::Priority(1000),
        rule::Nla::Protocol(2),
    ];
    let mut nl_hdr = NetlinkHeader::default();
    nl_hdr.flags = NLM_F_REQUEST | NLM_F_CREATE | NLM_F_EXCL | NLM_F_ACK;

    let mut msg = NetlinkMessage::new(
        nl_hdr,
        NetlinkPayload::from(RtnlMessage::NewRule(rule_msg)),
    );

    msg.finalize();
    let mut buf = vec![0; 1024 * 8];

    msg.serialize(&mut buf[..msg.buffer_len()]);

    println!(">>> {msg:?}");

    socket
        .send(&buf, 0)
        .expect("failed to send netlink message");

    let mut receive_buffer = vec![0; 4096];

    while let Ok(_size) = socket.recv(&mut receive_buffer, 0) {
        let bytes = &receive_buffer[..];
        let rx_packet = <NetlinkMessage<RtnlMessage>>::deserialize(bytes);
        println!("<<< {rx_packet:?}");
        if let Ok(rx_packet) = rx_packet {
            if let NetlinkPayload::Error(e) = rx_packet.payload {
                eprintln!("{e:?}");
            } else {
                return;
            }
        }
    }
}
