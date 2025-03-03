use futures::channel::mpsc::UnboundedSender;
use futures::future::Either;
use futures::stream::BoxStream;
use futures::{pin_mut, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, StreamExt};
use hex_literal::hex;
use rand::distributions::{Bernoulli, Distribution};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use rand_distr::Normal;
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

// Using static RNG seed to make tests reproducible between different runs
const RNG_SEED: [u8; 32] = hex!("d8a471f1c20490a3442b96fdde9d1807428096e1601b0cef0eea7e6d44a24c01");

pub async fn frames_send_and_recv<S>(
    num_frames: usize,
    frame_size: usize,
    alice: S,
    bob: S,
    timeout: Duration,
    alice_to_bob_only: bool,
    randomized_frame_sizes: bool,
) where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    #[derive(PartialEq, Eq)]
    enum Direction {
        Send,
        Recv,
        Both,
    }

    let frame_sizes = if randomized_frame_sizes {
        let norm_dist = rand_distr::Normal::new(frame_size as f64 * 0.75, frame_size as f64 / 4.0).unwrap();
        StdRng::from_seed(RNG_SEED)
            .sample_iter(norm_dist)
            .map(|s| (s as usize).max(10).min(2 * frame_size))
            .take(num_frames)
            .collect::<Vec<_>>()
    } else {
        std::iter::repeat(frame_size).take(num_frames).collect::<Vec<_>>()
    };

    let socket_worker = |mut socket: S, d: Direction| {
        let frame_sizes = frame_sizes.clone();
        let frame_sizes_total = frame_sizes.iter().sum();
        async move {
            let mut received = Vec::with_capacity(frame_sizes_total);
            let mut sent = Vec::with_capacity(frame_sizes_total);

            if d == Direction::Send || d == Direction::Both {
                for frame_size in &frame_sizes {
                    let mut write = vec![0u8; *frame_size];
                    hopr_crypto_random::random_fill(&mut write);
                    socket.write(&write).await?;
                    sent.extend(write);
                }
            }

            if d == Direction::Recv || d == Direction::Both {
                // Either read everything or timeout trying
                while received.len() < frame_sizes_total {
                    let mut buffer = [0u8; 2048];
                    let read = socket.read(&mut buffer).await?;
                    received.extend(buffer.into_iter().take(read));
                }
            }

            // TODO: fix this so it works properly
            // We cannot close immediately as some ack/resends might be ongoing
            //socket.close().await.unwrap();

            Ok::<_, std::io::Error>((sent, received))
        }
    };

    let alice_worker = async_std::task::spawn(socket_worker(
        alice,
        if alice_to_bob_only {
            Direction::Send
        } else {
            Direction::Both
        },
    ));
    let bob_worker = async_std::task::spawn(socket_worker(
        bob,
        if alice_to_bob_only {
            Direction::Recv
        } else {
            Direction::Both
        },
    ));

    let send_recv = futures::future::join(alice_worker, bob_worker);
    let timeout = async_std::task::sleep(timeout);

    pin_mut!(send_recv);
    pin_mut!(timeout);

    match futures::future::select(send_recv, timeout).await {
        Either::Left(((Ok((alice_sent, alice_recv)), Ok((bob_sent, bob_recv))), _)) => {
            assert_eq!(
                hex::encode(alice_sent),
                hex::encode(bob_recv),
                "alice sent must be equal to bob received"
            );
            assert_eq!(
                hex::encode(bob_sent),
                hex::encode(alice_recv),
                "bob sent must be equal to alice received",
            );
        }
        Either::Left(((Err(e), _), _)) => panic!("alice send recv error: {e}"),
        Either::Left(((_, Err(e)), _)) => panic!("bob send recv error: {e}"),
        Either::Right(_) => panic!("timeout"),
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

impl NetworkStats {
    pub fn assert_packets_sent(&self, expected: usize) {
        let actual = self.packets_sent.load(std::sync::atomic::Ordering::Relaxed);
        assert_eq!(
            actual,
            expected,
            "packets sent must be equal to {expected}, but was {actual}",
        );
    }

    pub fn assert_packets_received(&self, expected: usize) {
        let actual = self.packets_received.load(std::sync::atomic::Ordering::Relaxed);
        assert_eq!(
            actual,
            expected,
            "packets received must be equal to {expected}, but was {actual}",
        )
    }

    pub fn assert_bytes_sent(&self, expected: usize) {
        let actual = self.bytes_sent.load(std::sync::atomic::Ordering::Relaxed);
        assert_eq!(
            actual,
            expected,
            "bytes sent must be equal to {expected}, but was {actual}",
        );
    }

    pub fn assert_bytes_received(&self, expected: usize) {
        let actual = self.bytes_received.load(std::sync::atomic::Ordering::Relaxed);
        assert_eq!(
            actual,
            expected,
            "bytes received must be equal to {expected}, but was {actual}",
        );
    }
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
            stats
                .bytes_sent
                .fetch_add(buf.len(), std::sync::atomic::Ordering::Relaxed);
            stats.packets_sent.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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
                    stats
                        .bytes_received
                        .fetch_add(len, std::sync::atomic::Ordering::Relaxed);
                    stats
                        .packets_received
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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

/// Sample an index between `0` and `len - 1` using the given distribution and RNG.
pub fn sample_index<T: Distribution<f64>, R: Rng>(dist: &mut T, rng: &mut R, len: usize) -> usize {
    let f: f64 = dist.sample(rng);
    (f.max(0.0).round() as usize).min(len - 1)
}

/// Shuffles the given `vec` by taking the next element with index `|N(0,factor^2)`|, where
/// `N` denotes normal distribution.
/// When used on frame segments vector, it will shuffle the segments in a controlled manner;
/// such that an entire frame can unlikely swap position with another, if `factor` ~ frame length in segments.
pub fn linear_half_normal_shuffle<T, R: Rng>(rng: &mut R, mut vec: VecDeque<T>, factor: f64) -> Vec<T> {
    if factor == 0.0 || vec.is_empty() {
        return vec.into(); // no mixing
    }

    let mut dist = Normal::new(0.0, factor).unwrap();
    let mut ret = Vec::new();
    while !vec.is_empty() {
        ret.push(vec.remove(sample_index(&mut dist, rng, vec.len())).unwrap());
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::utils::test::{FaultyNetwork, FaultyNetworkConfig};
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
