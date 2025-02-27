use crate::IfEvent;
use async_io::Timer;
use futures::stream::{FusedStream, Stream};
use if_addrs::IfAddr;
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use std::collections::{HashSet, VecDeque};
use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

#[cfg(feature = "tokio")]
pub mod tokio {
    //! An interface watcher.
    //! **On this platform there is no difference between `tokio` and `smol` features,**
    //! **this was done to maintain the api compatible with other platforms**.

    /// Watches for interface changes.
    pub type IfWatcher = super::IfWatcher;
}

#[cfg(feature = "smol")]
pub mod smol {
    //! An interface watcher.
    //! **On this platform there is no difference between `tokio` and `smol` features,**
    //! **this was done to maintain the api compatible with other platforms**.

    /// Watches for interface changes.
    pub type IfWatcher = super::IfWatcher;
}

/// An address set/watcher
#[derive(Debug)]
pub struct IfWatcher {
    addrs: HashSet<IpNet>,
    queue: VecDeque<IfEvent>,
    ticker: Timer,
}

impl IfWatcher {
    /// Create a watcher.
    pub fn new() -> Result<Self> {
        Ok(Self {
            addrs: Default::default(),
            queue: Default::default(),
            ticker: Timer::interval_at(Instant::now(), Duration::from_secs(10)),
        })
    }

    fn resync(&mut self) -> Result<()> {
        let addrs = if_addrs::get_if_addrs()?;
        for old_addr in self.addrs.clone() {
            if !addrs.iter().any(|addr| addr.ip() == old_addr.addr()) {
                self.addrs.remove(&old_addr);
                self.queue.push_back(IfEvent::Down(old_addr));
            }
        }
        for new_addr in addrs {
            let ipnet = ifaddr_to_ipnet(new_addr.addr);
            if self.addrs.insert(ipnet) {
                self.queue.push_back(IfEvent::Up(ipnet));
            }
        }
        Ok(())
    }

    /// Iterate over current networks.
    pub fn iter(&self) -> impl Iterator<Item = &IpNet> {
        self.addrs.iter()
    }

    /// Poll for an address change event.
    pub fn poll_if_event(&mut self, cx: &mut Context) -> Poll<Result<IfEvent>> {
        loop {
            if let Some(event) = self.queue.pop_front() {
                return Poll::Ready(Ok(event));
            }
            if Pin::new(&mut self.ticker).poll_next(cx).is_pending() {
                return Poll::Pending;
            }
            if let Err(err) = self.resync() {
                return Poll::Ready(Err(err));
            }
        }
    }
}

impl Stream for IfWatcher {
    type Item = Result<IfEvent>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::into_inner(self).poll_if_event(cx).map(Some)
    }
}

impl FusedStream for IfWatcher {
    fn is_terminated(&self) -> bool {
        false
    }
}

fn ifaddr_to_ipnet(addr: IfAddr) -> IpNet {
    match addr {
        IfAddr::V4(ip) => {
            let prefix_len = (!u32::from_be_bytes(ip.netmask.octets())).leading_zeros();
            IpNet::V4(
                Ipv4Net::new(ip.ip, prefix_len as u8).expect("if_addrs returned a valid prefix"),
            )
        }
        IfAddr::V6(ip) => {
            let prefix_len = (!u128::from_be_bytes(ip.netmask.octets())).leading_zeros();
            IpNet::V6(
                Ipv6Net::new(ip.ip, prefix_len as u8).expect("if_addrs returned a valid prefix"),
            )
        }
    }
}
