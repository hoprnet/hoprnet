use crate::{server::Server, traits::DuplexStream};
use core::{
    hash::{Hash, Hasher},
    pin::Pin,
    task::{Context, Poll},
};
use futures::{stream::FusedStream, Future, Sink, Stream};
use std::{cell::RefCell, cmp::Ordering, collections::HashMap, rc::Rc};
use utils_log::error;

use libp2p::PeerId;

#[cfg(feature = "wasm")]
use wasm_bindgen_futures::spawn_local;

#[cfg(not(feature = "wasm"))]
use async_std::task::spawn_local;

struct RelayConnection<St> {
    conn_a: Server<St>,
    buffered_a: Option<Box<[u8]>>,
    conn_b: Server<St>,
    buffered_b: Option<Box<[u8]>>,
}

#[derive(Copy, Clone, Eq)]
struct RelayConnectionIdentifier {
    id_a: PeerId,
    id_b: PeerId,
}

impl<'a, St> RelayConnection<St> {
    fn new(conn_a: Server<St>, conn_b: Server<St>) -> Self {
        Self {
            conn_a,
            buffered_a: None,
            conn_b,
            buffered_b: None,
        }
    }
}

impl<St: DuplexStream> RelayConnection<St> {
    fn try_start_send_a(&mut self, cx: &mut Context<'_>, item: Box<[u8]>) -> Poll<Result<(), String>> {
        match Pin::new(&mut self.conn_a).poll_ready(cx)? {
            Poll::Ready(()) => Poll::Ready(Pin::new(&mut self.conn_a).start_send(item)),
            Poll::Pending => {
                self.buffered_a = Some(item);
                Poll::Pending
            }
        }
    }

    fn try_start_send_b(&mut self, cx: &mut Context<'_>, item: Box<[u8]>) -> Poll<Result<(), String>> {
        match Pin::new(&mut self.conn_b).poll_ready(cx)? {
            Poll::Ready(()) => Poll::Ready(Pin::new(&mut self.conn_b).start_send(item)),
            Poll::Pending => {
                self.buffered_b = Some(item);
                Poll::Pending
            }
        }
    }
}

impl<St: DuplexStream> Future for RelayConnection<St> {
    type Output = Result<(), String>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;

        let mut a_pending = false;

        if let Some(item) = this.buffered_a.take() {
            match this.try_start_send_a(cx, item) {
                Poll::Ready(Ok(())) => (),
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => a_pending = true,
            };
        }

        if let Some(item) = this.buffered_b.take() {
            match this.try_start_send_b(cx, item) {
                Poll::Ready(Ok(())) => (),
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => {
                    if a_pending {
                        return Poll::Pending;
                    }
                }
            };
        }

        let mut a_next_pending: bool;

        loop {
            a_next_pending = false;

            if this.conn_a.is_terminated() && this.conn_b.is_terminated() {
                break Poll::Ready(Ok(()));
            }

            if !this.conn_a.is_terminated() {
                match Pin::new(&mut this.conn_a).poll_next(cx) {
                    Poll::Ready(Some(item)) => match this.try_start_send_a(cx, item.unwrap()) {
                        Poll::Ready(Ok(())) => (),
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                        Poll::Pending => {
                            a_next_pending = true;
                        }
                    },
                    Poll::Ready(None) => match Pin::new(&mut this.conn_a).poll_flush(cx) {
                        Poll::Ready(Ok(())) => (),
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                        Poll::Pending => {
                            a_next_pending = true;
                        }
                    },
                    Poll::Pending => a_next_pending = true,
                };
            }

            if !this.conn_b.is_terminated() {
                match Pin::new(&mut this.conn_b).poll_next(cx) {
                    Poll::Ready(Some(item)) => match this.try_start_send_b(cx, item.unwrap()) {
                        Poll::Ready(Ok(())) => (),
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                        Poll::Pending => {
                            if a_next_pending {
                                return Poll::Pending;
                            }
                        }
                    },
                    Poll::Ready(None) => match Pin::new(&mut this.conn_b).poll_flush(cx) {
                        Poll::Ready(Ok(())) => (),
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                        Poll::Pending => {
                            if a_next_pending {
                                return Poll::Pending;
                            }
                        }
                    },
                    Poll::Pending => {
                        if a_next_pending {
                            return Poll::Pending;
                        }
                    }
                }
            }
        }
    }
}

impl ToString for RelayConnectionIdentifier {
    fn to_string(&self) -> String {
        format!("{}:{}", self.id_a.to_string(), self.id_b.to_string())
    }
}

impl Hash for RelayConnectionIdentifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id_a.hash(state);
        self.id_b.hash(state);
    }
}

impl PartialEq for RelayConnectionIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.id_a.eq(&other.id_a) && self.id_b.eq(&other.id_b)
    }
}

impl TryFrom<(PeerId, PeerId)> for RelayConnectionIdentifier {
    type Error = String;
    fn try_from(val: (PeerId, PeerId)) -> Result<Self, Self::Error> {
        match val.0.cmp(&val.1) {
            Ordering::Equal => Err("Keys must not be equal".into()),
            Ordering::Less => Ok(RelayConnectionIdentifier {
                id_a: val.0,
                id_b: val.1,
            }),
            Ordering::Greater => Ok(RelayConnectionIdentifier {
                id_a: val.1,
                id_b: val.0,
            }),
        }
    }
}

fn get_key_and_value<'a, St>(
    id_a: PeerId,
    conn_a: Server<St>,
    id_b: PeerId,
    conn_b: Server<St>,
) -> Result<(RelayConnectionIdentifier, RelayConnection<St>), String> {
    match id_a.cmp(&id_b) {
        Ordering::Equal => Err("Keys must not be equal".into()),
        Ordering::Less => Ok((
            RelayConnectionIdentifier { id_a, id_b },
            RelayConnection::new(conn_a, conn_b),
        )),
        Ordering::Greater => Ok((
            RelayConnectionIdentifier { id_a: id_b, id_b: id_a },
            RelayConnection::new(conn_b, conn_a),
        )),
    }
}

struct RelayConnections<St> {
    conns: Rc<RefCell<HashMap<RelayConnectionIdentifier, RelayConnection<St>>>>,
}

impl<St> ToString for RelayConnections<St> {
    fn to_string(&self) -> String {
        let prefix: String = "RelayConnections:\n".into();

        let items: String = self
            .conns
            .borrow()
            .keys()
            .map(|x| format!("  {}\n", x.to_string()))
            .collect();

        format!("{}{}", prefix, items)
    }
}

impl<St: DuplexStream + 'static> RelayConnections<St> {
    fn create_new(&'static mut self, id_a: PeerId, stream_a: St, id_b: PeerId, stream_b: St) -> Result<(), String> {
        let id1: RelayConnectionIdentifier = (id_a, id_b).try_into().unwrap();

        let bar = self.conns.as_ref();

        let conn_a = Server::new(
            stream_a,
            Box::new(move || {
                bar.borrow_mut().remove(&id1);
            }),
            Box::new(move || {
                bar.borrow_mut().remove(&id1);
            }),
        );

        let conn_b = Server::new(
            stream_b,
            Box::new(move || {
                bar.borrow_mut().remove(&id1);
            }),
            Box::new(move || {
                bar.borrow_mut().remove(&id1);
            }),
        );

        let (id, conn) = get_key_and_value(id_a, conn_a, id_b, conn_b)?;

        let conn = bar.borrow_mut().insert(id, conn).unwrap();

        spawn_local(async {
            match conn.await {
                Ok(()) => (),
                Err(e) => {
                    let id1: RelayConnectionIdentifier = (PeerId::random(), PeerId::random()).try_into().unwrap();

                    bar.borrow_mut().remove(&id1);
                    // self.conns.remove_entry(&id);
                    error!("{}", e)
                }
            }
        });

        Ok(())
    }

    // async isActive(source: PeerId, destination: PeerId, timeout?: number): Promise<boolean> {

    async fn is_active(&mut self, source: PeerId, destination: PeerId, maybe_timeout: Option<u64>) -> bool {
        let id: RelayConnectionIdentifier = match (source, destination).try_into() {
            Ok(x) => x,
            Err(e) => {
                error!("{}", e);
                return false;
            }
        };

        let mut conns = self.conns.borrow_mut();

        let relay_conn = match conns.get_mut(&id) {
            Some(x) => x,
            None => {
                error!("Connection does not exist");
                return false;
            }
        };

        match source.cmp(&destination) {
            Ordering::Equal => panic!("must not happen"),
            Ordering::Greater => {
                return match relay_conn.conn_b.ping(maybe_timeout).await {
                    Ok(_) => true,
                    Err(e) => {
                        error!("{}", e);
                        false
                    }
                }
            }
            Ordering::Less => {
                return match relay_conn.conn_a.ping(maybe_timeout).await {
                    Ok(_) => true,
                    Err(e) => {
                        error!("{}", e);
                        false
                    }
                }
            }
        }
    }

    fn update_existing(&mut self, source: PeerId, destination: PeerId, to_source: St) -> bool {
        let id: RelayConnectionIdentifier = match (source, destination).try_into() {
            Ok(x) => x,
            Err(e) => {
                error!("{}", e);
                return false;
            }
        };

        let mut conns = self.conns.borrow_mut();

        let relay_conn = match conns.get_mut(&id) {
            Some(x) => x,
            None => {
                error!("Connection does not exist");
                return false;
            }
        };

        match source.cmp(&destination) {
            Ordering::Equal => panic!("must not happen"),
            Ordering::Greater => {
                return match relay_conn.conn_b.update(to_source) {
                    Ok(_) => true,
                    Err(e) => {
                        error!("{}", e);
                        false
                    }
                }
            }
            Ordering::Less => {
                return match relay_conn.conn_a.update(to_source) {
                    Ok(_) => true,
                    Err(e) => {
                        error!("{}", e);
                        false
                    }
                }
            }
        }
    }
}

// #[cfg(feature = "wasm")]
// pub mod wasm {
//     struct
// }
