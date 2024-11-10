use async_std::prelude::FutureExt;
use futures::{AsyncReadExt, AsyncWriteExt, StreamExt, TryStreamExt};
use hopr_network_types::prelude::*;
use std::time::Duration;

#[test_log::test(async_std::test)]
async fn test_segmenter_reconstructor() -> anyhow::Result<()> {
    let (mut data_in, segments_out) = Segmenter::<462>::new(1500, 1024);
    let (segments_in, data_out) = frame_reconstructor(Duration::from_secs(10), 1024);

    let jh = async_std::task::spawn(segments_out.map(Ok).forward(segments_in));

    let data_written = hopr_crypto_random::random_bytes::<9001>();

    let data_read = async_std::task::spawn(async move {
        let mut out = Vec::new();
        let mut data_out = data_out
            .inspect(|frame| tracing::debug!("{:?}", frame))
            .map_err(std::io::Error::other)
            .into_async_read();

        data_out.read_to_end(&mut out).await?;
        Ok::<_, std::io::Error>(out)
    })
    .timeout(Duration::from_secs(5));

    data_in.write_all(&data_written).await?;
    data_in.close().await?;

    let data_read = data_read.await??;
    jh.await?;

    assert_eq!(&data_written, data_read.as_slice());

    Ok(())
}
