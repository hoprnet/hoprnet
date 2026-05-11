mod common;

use common::{
    random_packet_of_size, random_packets_of_count, send_and_receive_packets, send_relay_receive_channel_of_n_peers,
};
use hopr_protocol_app::prelude::{ApplicationData, ReservedTag};
use rstest::rstest;
use serial_test::serial;

#[rstest]
#[case::three_peers(3, 5)]
#[case::four_peers(4, 3)]
#[case::five_peers(5, 5)]
#[serial]
#[test_log::test(tokio::test)]
async fn packet_relayer_workflow(#[case] peer_count: usize, #[case] packet_count: usize) -> anyhow::Result<()> {
    let packets = random_packets_of_count(packet_count);
    send_relay_receive_channel_of_n_peers(peer_count, packets).await
}

#[serial]
#[test_log::test(tokio::test)]
async fn single_packet_through_3_peers() -> anyhow::Result<()> {
    let packets = random_packets_of_count(1);

    send_relay_receive_channel_of_n_peers(3, packets).await
}

#[serial]
#[test_log::test(tokio::test)]
async fn small_payload_packet_delivered_correctly() -> anyhow::Result<()> {
    let packet = ApplicationData::new(42u64, b"hi")?;

    let (received, _ticket_channels, _processes) = send_and_receive_packets(3, std::slice::from_ref(&packet)).await?;

    assert_eq!(received.len(), 1);
    assert_eq!(received[0].1.data.plain_text, packet.plain_text);

    Ok(())
}

#[serial]
#[test_log::test(tokio::test)]
async fn large_payload_near_max_size() -> anyhow::Result<()> {
    // ApplicationData::PAYLOAD_SIZE is the max; try a payload that's close to it
    let max_payload = ApplicationData::PAYLOAD_SIZE;
    let packet = random_packet_of_size(max_payload - 1);

    let (received, _ticket_channels, _processes) = send_and_receive_packets(3, std::slice::from_ref(&packet)).await?;

    assert_eq!(received.len(), 1);
    assert_eq!(received[0].1.data.plain_text, packet.plain_text);

    Ok(())
}

#[serial]
#[test_log::test(tokio::test)]
async fn many_packets_delivered_in_order_independent_of_content() -> anyhow::Result<()> {
    let packet_count = 20;
    let packets = random_packets_of_count(packet_count);

    let (received, _, _processes) = send_and_receive_packets(3, &packets).await?;

    assert_eq!(received.len(), packet_count);

    // Verify all sent packets are present in received (order may differ due to mixing)
    let mut sent_sorted: Vec<_> = packets.iter().map(|p| p.plain_text.clone()).collect();
    sent_sorted.sort();
    let mut recv_sorted: Vec<_> = received.iter().map(|(_, d)| d.data.plain_text.clone()).collect();
    recv_sorted.sort();

    assert_eq!(sent_sorted, recv_sorted);

    Ok(())
}

#[serial]
#[test_log::test(tokio::test)]
async fn identical_payloads_are_all_delivered() -> anyhow::Result<()> {
    // Send multiple packets with identical payloads — all should arrive
    let payload = b"identical payload data for dedup test";
    let packets: Vec<ApplicationData> = (0..5)
        .map(|_| ApplicationData::new(ReservedTag::UPPER_BOUND, payload.as_slice()))
        .collect::<Result<_, _>>()?;

    let (received, _, _processes) = send_and_receive_packets(3, &packets).await?;

    assert_eq!(received.len(), 5);
    assert!(
        received
            .iter()
            .all(|(_, data)| data.data.plain_text.as_ref() == payload.as_slice())
    );

    Ok(())
}
