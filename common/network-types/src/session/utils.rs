use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use futures::channel::mpsc::UnboundedSender;
use futures::stream::BoxStream;
use futures::{AsyncRead, AsyncWrite, StreamExt};
use rand::distributions::Bernoulli;
use rand::prelude::{thread_rng, Distribution, Rng, SeedableRng, StdRng};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RetryToken {
    pub num_retry: usize,
    pub started_at: Instant,
    backoff_base: f64,
    created_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RetryResult {
    Wait(Duration),
    RetryNow(RetryToken),
    Expired,
}

impl RetryToken {
    pub fn new(now: Instant, backoff_base: f64) -> Self {
        Self {
            num_retry: 0,
            started_at: now,
            created_at: Instant::now(),
            backoff_base,
        }
    }

    pub fn replenish(self, now: Instant, backoff_base: f64) -> Self {
        Self {
            num_retry: 0,
            started_at: now,
            created_at: self.created_at,
            backoff_base,
        }
    }

    fn retry_in(&self, base: Duration, max_duration: Duration, jitter_dev: f64) -> Option<Duration> {
        let jitter_coeff = if jitter_dev > 0.0 {
            // Should not use jitter with sigma > 0.25
            rand_distr::Normal::new(1.0, jitter_dev.min(0.25))
                .unwrap()
                .sample(&mut thread_rng())
                .abs()
        } else {
            1.0
        };

        // jitter * base * backoff_base ^ num_retry
        let duration = base.mul_f64(jitter_coeff * self.backoff_base.powi(self.num_retry as i32));
        (duration < max_duration).then_some(duration)
    }

    pub fn check(&self, now: Instant, base: Duration, max: Duration, jitter_dev: f64) -> RetryResult {
        match self.retry_in(base, max, jitter_dev) {
            None => RetryResult::Expired,
            Some(retry_in) if self.started_at + retry_in >= now => RetryResult::Wait(self.started_at + retry_in - now),
            _ => RetryResult::RetryNow(Self {
                num_retry: self.num_retry + 1,
                started_at: self.started_at,
                backoff_base: self.backoff_base,
                created_at: self.created_at,
            }),
        }
    }

    pub fn time_since_creation(&self) -> Duration {
        self.created_at.elapsed()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FaultyNetworkConfig {
    pub fault_prob: f64,
    pub mixing_factor: usize,
    pub rng_seed: [u8; 32],
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
            rng_seed: [
                0xd8, 0xa4, 0x71, 0xf1, 0xc2, 0x04, 0x90, 0xa3, 0x44, 0x2b, 0x96, 0xfd, 0xde, 0x9d, 0x18, 0x07, 0x42,
                0x80, 0x96, 0xe1, 0x60, 0x1b, 0x0c, 0xef, 0x0e, 0xea, 0x7e, 0x6d, 0x44, 0xa2, 0x4c, 0x01,
            ],
        }
    }
}

/// Network simulator used for testing.
pub struct FaultyNetwork<'a, const C: usize> {
    ingress: UnboundedSender<Box<[u8]>>,
    egress: BoxStream<'a, Box<[u8]>>,
    stats: Option<NetworkStats>,
}

impl<const C: usize> AsyncWrite for FaultyNetwork<'_, C> {
    fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        if buf.len() > C {
            return Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("data length passed to downstream must be less or equal to {C}"),
            )));
        }

        if let Err(e) = self.ingress.unbounded_send(buf.into()) {
            return Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                format!("failed to send data: {e}"),
            )));
        }

        if let Some(stats) = &self.stats {
            stats.bytes_sent.fetch_add(buf.len(), Ordering::Relaxed);
            stats.packets_sent.fetch_add(1, Ordering::Relaxed);
        }

        tracing::trace!("FaultyNetwork::poll_write {} bytes", buf.len());
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.ingress.close_channel();
        Poll::Ready(Ok(()))
    }
}

impl<const C: usize> AsyncRead for FaultyNetwork<'_, C> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        match self.egress.poll_next_unpin(cx) {
            Poll::Ready(Some(item)) => {
                let len = buf.len().min(item.len());
                buf[..len].copy_from_slice(&item.as_ref()[..len]);

                if let Some(stats) = &self.stats {
                    stats.bytes_received.fetch_add(len, Ordering::Relaxed);
                    stats.packets_received.fetch_add(1, Ordering::Relaxed);
                }

                tracing::trace!("FaultyNetwork::poll_read: {len} bytes ready");
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
        let egress = egress.filter(move |_| futures::future::ready(bernoulli.sample(&mut rng)));

        let egress = if cfg.mixing_factor > 0 {
            let mut rng = StdRng::from_seed(cfg.rng_seed);
            egress
                .map(move |e| {
                    let wait = rng.gen_range(0..20);
                    async move {
                        hopr_async_runtime::prelude::sleep(Duration::from_micros(wait)).await;
                        e
                    }
                })
                .buffer_unordered(cfg.mixing_factor)
                .boxed()
        } else {
            egress.boxed()
        };

        Self { ingress, egress, stats }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::io::{AsyncReadExt, AsyncWriteExt};
    use std::future::Future;

    fn spawn_single_byte_read_write<C>(
        channel: C,
        data: Vec<u8>,
    ) -> (impl Future<Output = Vec<u8>>, impl Future<Output = Vec<u8>>)
    where
        C: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let (mut recv, mut send) = channel.split();

        let len = data.len();
        let read = async_std::task::spawn(async move {
            let mut out = Vec::with_capacity(len);
            for _ in 0..len {
                let mut bytes = [0u8; 1];
                if recv.read(&mut bytes).await.unwrap() > 0 {
                    out.push(bytes[0]);
                } else {
                    break;
                }
            }
            out
        });

        let written = async_std::task::spawn(async move {
            let mut out = Vec::with_capacity(len);
            for byte in data {
                send.write(&[byte]).await.unwrap();
                out.push(byte);
            }
            send.close().await.unwrap();
            out
        });

        (read, written)
    }

    #[async_std::test]
    async fn faulty_network_mixing() {
        const MIX_FACTOR: usize = 2;
        const COUNT: usize = 20;

        let net = FaultyNetwork::<466>::new(
            FaultyNetworkConfig {
                mixing_factor: MIX_FACTOR,
                ..Default::default()
            },
            None,
        );

        let (read, written) = spawn_single_byte_read_write(net, (0..COUNT as u8).collect());
        let (read, _) = futures::future::join(read, written).await;

        for (pos, value) in read.into_iter().enumerate() {
            assert!(
                pos.abs_diff(value as usize) <= MIX_FACTOR,
                "packet must not be off from its position by more than then mixing factor"
            );
        }
    }

    #[async_std::test]
    async fn faulty_network_packet_drop() {
        const DROP: f64 = 0.3333;
        const COUNT: usize = 20;

        let net = FaultyNetwork::<466>::new(
            FaultyNetworkConfig {
                fault_prob: DROP,
                ..Default::default()
            },
            None,
        );

        let (read, written) = spawn_single_byte_read_write(net, (0..COUNT as u8).collect());
        let (read, written) = futures::future::join(read, written).await;

        let max_drop = (written.len() as f64 * (1.0 - DROP) - 2.0).floor() as usize;
        assert!(read.len() >= max_drop, "dropped more than {max_drop}: {}", read.len());
    }

    #[async_std::test]
    async fn faulty_network_reliable() {
        const COUNT: usize = 20;

        let net = FaultyNetwork::<466>::new(Default::default(), None);

        let (read, written) = spawn_single_byte_read_write(net, (0..COUNT as u8).collect());
        let (read, written) = futures::future::join(read, written).await;

        assert_eq!(read, written);
    }
}
