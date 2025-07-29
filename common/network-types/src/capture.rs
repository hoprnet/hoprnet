//! Contains adapter for [`futures::io::AsyncRead`] and [`futures::io::AsyncWrite`] that writes into a Pcap file.
//!
//! Requires the `capture` feature to be enabled.

use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::{StreamExt, pin_mut};
use pcap_file::{
    DataLink,
    pcapng::blocks::{
        enhanced_packet::{EnhancedPacketBlock, EnhancedPacketOption},
        interface_description::{InterfaceDescriptionBlock, InterfaceDescriptionOption},
    },
};

#[derive(Clone, Debug, strum::Display)]
enum CapturedPacket {
    Incoming(std::time::Duration, Box<[u8]>),
    Outgoing(std::time::Duration, Box<[u8]>),
}

impl AsRef<[u8]> for CapturedPacket {
    fn as_ref(&self) -> &[u8] {
        match self {
            CapturedPacket::Incoming(_, data) => data.as_ref(),
            CapturedPacket::Outgoing(_, data) => data.as_ref(),
        }
    }
}

impl CapturedPacket {
    fn timestamp(&self) -> std::time::Duration {
        match self {
            CapturedPacket::Incoming(timestamp, _) => *timestamp,
            CapturedPacket::Outgoing(timestamp, _) => *timestamp,
        }
    }
}

/// Adapter for [`futures::io::AsyncRead`] and [`futures::io::AsyncWrite`] that writes into a Pcap file.
#[pin_project::pin_project]
pub struct PcapIO<T> {
    #[pin]
    inner: T,
    write_closed: bool,
    read_closed: bool,
    sender: futures::channel::mpsc::Sender<CapturedPacket>,
}

impl<T> PcapIO<T> {
    fn new(inner: T, file: &str) -> Self {
        let (sender, receiver) = futures::channel::mpsc::channel::<CapturedPacket>(10_000);
        let file = file.to_owned();

        hopr_async_runtime::prelude::spawn(async move {
            match std::fs::File::create(file)
                .and_then(|f| pcap_file::pcapng::PcapNgWriter::new(f).map_err(std::io::Error::other))
            {
                Ok(mut writer) => {
                    let _ = writer.write_pcapng_block(InterfaceDescriptionBlock {
                        linktype: DataLink::USER1,
                        snaplen: 0,
                        options: vec![InterfaceDescriptionOption::IfTsResol(0x09)],
                    });

                    let writer = Arc::new(std::sync::Mutex::new(writer));
                    pin_mut!(receiver);
                    while let Some(next) = receiver.next().await {
                        let writer = writer.clone();

                        if let Err(error) = hopr_async_runtime::prelude::spawn_blocking(move || {
                            let data = next.as_ref();
                            writer
                                .lock()
                                .map_err(|_| std::io::Error::other("lock error"))
                                .and_then(|mut writer| {
                                    writer
                                        .write_pcapng_block(EnhancedPacketBlock {
                                            interface_id: 0,
                                            timestamp: next.timestamp(),
                                            original_len: data.len() as u32,
                                            data: data.into(),
                                            options: vec![EnhancedPacketOption::Comment(next.to_string().into())],
                                        })
                                        .map_err(std::io::Error::other)
                                })
                        })
                        .await
                        {
                            tracing::error!(%error, "error writing to pcap file");
                            break;
                        };
                    }
                }
                Err(error) => tracing::error!(%error, "failed to create pcap writer"),
            }
        });

        Self {
            inner,
            sender,
            write_closed: false,
            read_closed: false,
        }
    }
}

impl<T: futures::io::AsyncWrite> futures::io::AsyncWrite for PcapIO<T> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        let this = self.project();
        // If the channel is full, drop the data
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        let _ = this.sender.try_send(CapturedPacket::Outgoing(timestamp, buf.into()));
        this.inner.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let this = self.project();
        *this.write_closed = true;
        if *this.read_closed {
            this.sender.close_channel();
        }
        this.inner.poll_close(cx)
    }
}

impl<T: futures::io::AsyncRead> futures::io::AsyncRead for PcapIO<T> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        let this = self.project();
        match futures::ready!(this.inner.poll_read(cx, buf)) {
            Ok(read) if read > 0 => {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap();
                let _ = this
                    .sender
                    .try_send(CapturedPacket::Incoming(timestamp, Box::from(&buf[0..read])));
                Poll::Ready(Ok(read))
            }
            Ok(_) => {
                *this.read_closed = true;
                if *this.write_closed {
                    this.sender.close_channel();
                }
                Poll::Ready(Ok(0))
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

pub trait PcapIoExt: futures::io::AsyncWrite + futures::io::AsyncRead {
    /// Wraps the [`futures::io::AsyncRead`] and [`futures::io::AsyncWrite`] object
    /// writing the data into the specified `pcap` file and passing them on.
    fn capture(self, file: &str) -> PcapIO<Self>
    where
        Self: Sized,
    {
        PcapIO::new(self, file)
    }
}

impl<T: ?Sized> PcapIoExt for T where T: futures::io::AsyncWrite + futures::io::AsyncRead {}
