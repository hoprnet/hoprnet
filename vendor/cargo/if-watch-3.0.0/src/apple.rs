use crate::{IfEvent, IpNet, Ipv4Net, Ipv6Net};
use core_foundation::array::CFArray;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_foundation::string::CFString;
use fnv::FnvHashSet;
use futures::channel::mpsc;
use futures::stream::{FusedStream, Stream};
use if_addrs::IfAddr;
use std::collections::VecDeque;
use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll};
use system_configuration::dynamic_store::{
    SCDynamicStore, SCDynamicStoreBuilder, SCDynamicStoreCallBackContext,
};

#[cfg(feature = "tokio")]
pub mod tokio {
    //! An interface watcher.
    //! **On Apple Platforms there is no difference between `tokio` and `smol` features,**
    //! **this was done to maintain the api compatible with other platforms**.

    /// Watches for interface changes.
    pub type IfWatcher = super::IfWatcher;
}

#[cfg(feature = "smol")]
pub mod smol {
    //! An interface watcher.
    //! **On Apple platforms there is no difference between `tokio` and `smol` features,**
    //! **this was done to maintain the api compatible with other platforms**.

    /// Watches for interface changes.
    pub type IfWatcher = super::IfWatcher;
}

#[derive(Debug)]
pub struct IfWatcher {
    addrs: FnvHashSet<IpNet>,
    queue: VecDeque<IfEvent>,
    rx: mpsc::Receiver<()>,
}

impl IfWatcher {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel(1);
        std::thread::spawn(|| background_task(tx));
        let mut watcher = Self {
            addrs: Default::default(),
            queue: Default::default(),
            rx,
        };
        watcher.resync()?;
        Ok(watcher)
    }

    fn resync(&mut self) -> Result<()> {
        let addrs = if_addrs::get_if_addrs()?;
        for old_addr in self.addrs.clone() {
            if addrs
                .iter()
                .find(|addr| addr.ip() == old_addr.addr())
                .is_none()
            {
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
            if Pin::new(&mut self.rx).poll_next(cx).is_pending() {
                return Poll::Pending;
            }
            if let Err(error) = self.resync() {
                return Poll::Ready(Err(error));
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

fn callback(_store: SCDynamicStore, _changed_keys: CFArray<CFString>, info: &mut mpsc::Sender<()>) {
    match info.try_send(()) {
        Err(err) if err.is_disconnected() => CFRunLoop::get_current().stop(),
        _ => {}
    }
}

fn background_task(tx: mpsc::Sender<()>) {
    let store = SCDynamicStoreBuilder::new("global-network-watcher")
        .callback_context(SCDynamicStoreCallBackContext {
            callout: callback,
            info: tx,
        })
        .build();
    store.set_notification_keys(
        &CFArray::<CFString>::from_CFTypes(&[]),
        &CFArray::from_CFTypes(&[CFString::new("State:/Network/Interface/.*/IPv.")]),
    );
    let source = store.create_run_loop_source();
    let run_loop = CFRunLoop::get_current();
    run_loop.add_source(&source, unsafe { kCFRunLoopCommonModes });
    CFRunLoop::run_current();
}
