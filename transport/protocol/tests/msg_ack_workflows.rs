mod common;

use common::packet_relayer_workflow_n_peers;
use serial_test::serial;

#[serial]
#[async_std::test]
// #[tracing_test::traced_test]
async fn test_packet_relayer_workflow_3_peers() -> anyhow::Result<()> {
    packet_relayer_workflow_n_peers(3, 5).await
}

#[serial]
#[async_std::test]
// #[tracing_test::traced_test]
async fn test_packet_relayer_workflow_5_peers() -> anyhow::Result<()> {
    packet_relayer_workflow_n_peers(5, 5).await
}
