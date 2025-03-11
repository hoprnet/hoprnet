mod common;

use common::{random_packets_of_count, send_relay_receive_channel_of_n_peers};
use serial_test::serial;

#[serial]
#[async_std::test]
// #[tracing_test::traced_test]
async fn test_packet_relayer_workflow_3_peers() -> anyhow::Result<()> {
    let packets = random_packets_of_count(5);

    send_relay_receive_channel_of_n_peers(3, packets).await
}

#[serial]
#[async_std::test]
// #[tracing_test::traced_test]
async fn test_packet_relayer_workflow_5_peers() -> anyhow::Result<()> {
    let packets = random_packets_of_count(5);

    send_relay_receive_channel_of_n_peers(5, packets).await
}
