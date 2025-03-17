//! IP address watching.
#![deny(missing_docs)]
#![deny(warnings)]

pub use ipnet::{IpNet, Ipv4Net, Ipv6Net};

#[cfg(target_os = "macos")]
mod apple;
#[cfg(target_os = "ios")]
mod apple;
#[cfg(not(any(
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
)))]
mod fallback;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod win;

#[cfg(any(target_os = "macos", target_os = "ios"))]
#[cfg(feature = "tokio")]
pub use apple::tokio;

#[cfg(any(target_os = "macos", target_os = "ios"))]
#[cfg(feature = "smol")]
pub use apple::smol;

#[cfg(feature = "smol")]
#[cfg(not(any(
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
)))]
pub use fallback::smol;

#[cfg(feature = "tokio")]
#[cfg(not(any(
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
)))]
pub use fallback::tokio;

#[cfg(target_os = "windows")]
#[cfg(feature = "tokio")]
pub use win::tokio;

#[cfg(target_os = "windows")]
#[cfg(feature = "smol")]
pub use win::smol;

#[cfg(target_os = "linux")]
#[cfg(feature = "tokio")]
pub use linux::tokio;

#[cfg(target_os = "linux")]
#[cfg(feature = "smol")]
pub use linux::smol;

/// An address change event.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IfEvent {
    /// A new local address has been added.
    Up(IpNet),
    /// A local address has been deleted.
    Down(IpNet),
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use std::pin::Pin;

    #[test]
    fn test_smol_ip_watch() {
        use super::smol::IfWatcher;

        smol::block_on(async {
            let mut set = IfWatcher::new().unwrap();
            let event = set.select_next_some().await.unwrap();
            println!("Got event {:?}", event);
        });
    }

    #[tokio::test]
    async fn test_tokio_ip_watch() {
        use super::tokio::IfWatcher;

        let mut set = IfWatcher::new().unwrap();
        let event = set.select_next_some().await.unwrap();
        println!("Got event {:?}", event);
    }

    #[test]
    fn test_smol_is_send() {
        use super::smol::IfWatcher;

        smol::block_on(async {
            fn is_send<T: Send>(_: T) {}
            is_send(IfWatcher::new());
            is_send(IfWatcher::new().unwrap());
            is_send(Pin::new(&mut IfWatcher::new().unwrap()));
        });
    }

    #[tokio::test]
    async fn test_tokio_is_send() {
        use super::tokio::IfWatcher;

        fn is_send<T: Send>(_: T) {}
        is_send(IfWatcher::new());
        is_send(IfWatcher::new().unwrap());
        is_send(Pin::new(&mut IfWatcher::new().unwrap()));
    }
}
