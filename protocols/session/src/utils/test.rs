use std::{
    collections::HashSet,
    convert::identity,
    pin::Pin,
    sync::{Arc, atomic::AtomicUsize},
    task::{Context, Poll},
    time::Duration,
};

use futures::{AsyncRead, AsyncReadExt, AsyncWrite, StreamExt, channel::mpsc::UnboundedSender, stream::BoxStream};
use hopr_network_types::utils::DuplexIO;
use rand::{RngExt, SeedableRng, prelude::StdRng};
use rand_distr::{Bernoulli, Distribution};
use tracing::instrument;

// Using static RNG seed to make tests reproducible between different runs
// const RNG_SEED: [u8; 32] = hex_literal::hex!("d8a471f1c20490a3442b96fdde9d1807428096e1601b0cef0eea7e6d44a24c01");

#[derive(Debug, Clone, PartialEq)]
pub struct FaultyNetworkConfig {
    pub fault_prob: f64,
    pub mixing_factor: usize,
    pub avg_delay: Duration,
    pub rng_seed: [u8; 32],
    pub ids_to_drop: HashSet<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct NetworkStats {
    pub packets_sent: Arc<AtomicUsize>,
    pub packets_received: Arc<AtomicUsize>,
    pub bytes_sent: Arc<AtomicUsize>,
    pub bytes_received: Arc<AtomicUsize>,
}

impl Default for FaultyNetworkConfig {
    fn default() -> Self {
        Self {
            fault_prob: 0.0,
            mixing_factor: 0,
            avg_delay: Duration::ZERO,
            rng_seed: [
                0xd8, 0xa4, 0x71, 0xf1, 0xc2, 0x04, 0x90, 0xa3, 0x44, 0x2b, 0x96, 0xfd, 0xde, 0x9d, 0x18, 0x07, 0x42,
                0x80, 0x96, 0xe1, 0x60, 0x1b, 0x0c, 0xef, 0x0e, 0xea, 0x7e, 0x6d, 0x44, 0xa2, 0x4c, 0x01,
            ],
            ids_to_drop: Default::default(),
        }
    }
}

/// Network simulator used for testing.
pub struct FaultyNetwork<'a, const C: usize> {
    ingress: UnboundedSender<Box<[u8]>>,
    egress: BoxStream<'a, Box<[u8]>>,
    stats: Option<NetworkStats>,
    packet_counter: AtomicUsize,
    ids_to_drop: HashSet<usize>,
}

impl<const C: usize> AsyncWrite for FaultyNetwork<'_, C> {
    #[instrument(name = "FaultyNetwork::poll_write", level = "trace", skip(self, buf), fields(len = buf.len()))]
    fn poll_write(self: Pin<&mut Self>, _: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        if buf.len() > C {
            return Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "data length passed to downstream must be less or equal to {C}, actual: {}",
                    buf.len()
                ),
            )));
        }

        let packet_id = self.packet_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if !self.ids_to_drop.contains(&packet_id) {
            if let Err(e) = self.ingress.unbounded_send(buf.into()) {
                return Poll::Ready(Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    format!("failed to send data: {e}"),
                )));
            }

            if let Some(stats) = &self.stats {
                stats
                    .bytes_sent
                    .fetch_add(buf.len(), std::sync::atomic::Ordering::Relaxed);
                stats.packets_sent.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        } else {
            tracing::trace!(packet_id, "packet intentionally dropped");
        }

        tracing::trace!(len = buf.len(), "wrote bytes");
        Poll::Ready(Ok(buf.len()))
    }

    #[instrument(name = "FaultyNetwork::poll_flush", level = "trace", skip(self))]
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        tracing::trace!("polling flush");
        Poll::Ready(Ok(()))
    }

    #[instrument(name = "FaultyNetwork::poll_close", level = "trace", skip(self))]
    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        tracing::trace!("polling close");
        self.ingress.close_channel();
        Poll::Ready(Ok(()))
    }
}

impl<const C: usize> AsyncRead for FaultyNetwork<'_, C> {
    #[instrument(name = "FaultyNetwork::poll_read", level = "trace", skip(self, cx, buf))]
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        match self.egress.poll_next_unpin(cx) {
            Poll::Ready(Some(item)) => {
                let len = buf.len().min(item.len());
                buf[..len].copy_from_slice(&item.as_ref()[..len]);

                if let Some(stats) = &self.stats {
                    stats
                        .bytes_received
                        .fetch_add(len, std::sync::atomic::Ordering::Relaxed);
                    stats
                        .packets_received
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }

                tracing::trace!(len, "bytes ready");
                Poll::Ready(Ok(len))
            }
            Poll::Ready(None) => Poll::Ready(Ok(0)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<const C: usize> FaultyNetwork<'_, C> {
    #[allow(dead_code)]
    pub fn new(cfg: FaultyNetworkConfig, stats: Option<NetworkStats>) -> Self {
        let (ingress, egress) = futures::channel::mpsc::unbounded::<Box<[u8]>>();

        let mut rng = StdRng::from_seed(cfg.rng_seed);
        let bernoulli = Bernoulli::new(1.0 - cfg.fault_prob).unwrap();
        let egress = egress
            .filter(move |_| futures::future::ready(bernoulli.sample(&mut rng)))
            .map(move |e| {
                let mut avg_delay = cfg.avg_delay;
                if cfg.mixing_factor > 0 {
                    avg_delay = avg_delay.max(Duration::from_micros(10));
                }

                let mut rng = StdRng::from_seed(cfg.rng_seed);
                let wait = if !avg_delay.is_zero() {
                    rng.random_range(Duration::ZERO..2 * avg_delay)
                } else {
                    Duration::ZERO
                };
                async move {
                    if wait > Duration::ZERO {
                        hopr_async_runtime::prelude::sleep(wait).await;
                    }
                    e
                }
            });

        let egress = if cfg.mixing_factor > 0 {
            egress.buffer_unordered(cfg.mixing_factor).boxed()
        } else {
            egress.then(identity).boxed()
        };

        Self {
            ingress,
            egress,
            stats,
            packet_counter: AtomicUsize::new(0),
            ids_to_drop: cfg.ids_to_drop,
        }
    }
}

type Duplex<'a, const MTU: usize> =
    DuplexIO<futures::io::WriteHalf<FaultyNetwork<'a, MTU>>, futures::io::ReadHalf<FaultyNetwork<'a, MTU>>>;

pub fn setup_alice_bob<'a, const MTU: usize>(
    network_cfg: FaultyNetworkConfig,
    alice_stats: Option<NetworkStats>,
    bob_stats: Option<NetworkStats>,
) -> (Duplex<'a, MTU>, Duplex<'a, MTU>) {
    let (alice_stats, bob_stats) = alice_stats
        .zip(bob_stats)
        .map(|(alice, bob)| {
            (
                NetworkStats {
                    packets_sent: bob.packets_sent,
                    bytes_sent: bob.bytes_sent,
                    packets_received: alice.packets_received,
                    bytes_received: alice.bytes_received,
                },
                NetworkStats {
                    packets_sent: alice.packets_sent,
                    bytes_sent: alice.bytes_sent,
                    packets_received: bob.packets_received,
                    bytes_received: bob.bytes_received,
                },
            )
        })
        .unzip();

    let (alice_reader, alice_writer) = FaultyNetwork::new(network_cfg.clone(), alice_stats).split();
    let (bob_reader, bob_writer) = FaultyNetwork::new(network_cfg, bob_stats).split();

    (DuplexIO(bob_writer, alice_reader), DuplexIO(alice_writer, bob_reader))
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use futures::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    async fn spawn_single_byte_read_write<C>(channel: C, data: Vec<u8>) -> anyhow::Result<(Vec<u8>, Vec<u8>)>
    where
        C: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let (mut recv, mut send) = channel.split();

        let len = data.len();
        let read = tokio::task::spawn(async move {
            let mut out = Vec::with_capacity(len);
            for _ in 0..len {
                let mut bytes = [0u8; 1];
                if recv.read(&mut bytes).await.context("read")? > 0 {
                    out.push(bytes[0]);
                } else {
                    break;
                }
            }
            anyhow::Ok(out)
        });

        let written = tokio::task::spawn(async move {
            let mut out = Vec::with_capacity(len);
            for byte in data {
                send.write_all(&[byte]).await.context("write")?;
                out.push(byte);
            }
            send.close().await.context("close")?;
            anyhow::Ok(out)
        });

        let (read, written) = futures::future::try_join(read, written).await?;

        Ok((read?, written?))
    }

    #[tokio::test]
    async fn faulty_network_mixing() -> anyhow::Result<()> {
        const MIX_FACTOR: usize = 2;
        const COUNT: usize = 20;

        let net = FaultyNetwork::<466>::new(
            FaultyNetworkConfig {
                mixing_factor: MIX_FACTOR,
                ..Default::default()
            },
            None,
        );

        let (read, _) = spawn_single_byte_read_write(net, (0..COUNT as u8).collect()).await?;

        for (pos, value) in read.into_iter().enumerate() {
            assert!(
                pos.abs_diff(value as usize) <= MIX_FACTOR,
                "packet must not be off from its position by more than then mixing factor"
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn faulty_network_packet_drop() -> anyhow::Result<()> {
        const DROP: f64 = 0.3333;
        const COUNT: usize = 20;

        let net = FaultyNetwork::<466>::new(
            FaultyNetworkConfig {
                fault_prob: DROP,
                ..Default::default()
            },
            None,
        );

        let (read, written) = spawn_single_byte_read_write(net, (0..COUNT as u8).collect()).await?;

        let max_drop = (written.len() as f64 * (1.0 - DROP) - 2.0).floor() as usize;
        assert!(read.len() >= max_drop, "dropped more than {max_drop}: {}", read.len());

        Ok(())
    }

    #[tokio::test]
    async fn faulty_network_reliable() -> anyhow::Result<()> {
        const COUNT: usize = 20;

        let net = FaultyNetwork::<466>::new(Default::default(), None);

        let (read, written) = spawn_single_byte_read_write(net, (0..COUNT as u8).collect()).await?;

        assert_eq!(read, written);

        Ok(())
    }
}
