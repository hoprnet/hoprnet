use async_oneshot::*;
use futures_lite::future::block_on;
use futures_micro::prelude::*;
use waker_fn::waker_fn;

#[test]
fn send_recv() {
    let (mut s,r) = oneshot::<i32>();
    assert_eq!(
        block_on(zip!(async { s.send(42).unwrap() }, r)),
        ((), Ok(42))
    )
}
#[test]
fn recv_send() {
    let (mut s,r) = oneshot::<i32>();
    assert_eq!(
        block_on(zip!(r, async { s.send(42).unwrap() })),
        (Ok(42), ())
    )
}

#[test]
fn recv_recv() {
    let (_s, mut r) = oneshot::<i32>();
    let waker = waker_fn(|| ());
    let mut ctx = Context::from_waker(&waker);
    assert_eq!(Receiver::poll(Pin::new(&mut r), &mut ctx), Poll::Pending);
    assert_eq!(Receiver::poll(Pin::new(&mut r), &mut ctx), Poll::Pending);
}

#[test]
fn close_recv() {
    let (s,r) = oneshot::<i32>();
    s.close();
    assert_eq!(Err(Closed()), block_on(r));
}

#[test]
fn close_send() {
    let (mut s,r) = oneshot::<bool>();
    r.close();
    assert_eq!(Err(Closed()), s.send(true));
}

#[test]
fn send_close() {
    let (mut s,r) = oneshot::<bool>();
    s.send(true).unwrap();
    r.close();
}

#[test]
fn recv_close() {
    let (s,r) = oneshot::<bool>();
    assert_eq!(
        block_on(zip!(r, async { s.close() })),
        (Err(Closed()), ())
    )
}

#[test]
fn wait_close() {
    let (s,r) = oneshot::<bool>();
    assert_eq!(
        block_on(
            zip!(async { s.wait().await.unwrap_err() },
                 async { r.close() })
        ),
        (Closed(), ())
    )
}

#[test]
fn wait_wait() {
    let (s, _r) = oneshot::<i32>();
    let waker = waker_fn(|| ());
    let mut ctx = Context::from_waker(&waker);
    let mut wait = s.wait();
    assert!(Future::poll(Pin::new(&mut wait), &mut ctx).is_pending());
    assert!(Future::poll(Pin::new(&mut wait), &mut ctx).is_pending());
}


#[test]
fn wait_recv_close() {
    let (s,r) = oneshot::<bool>();
    assert_eq!(
        block_on(
            zip!(async { s.wait().await.unwrap().close(); println!("closed"); }, r)
        ),
        ((), Err(Closed()))
    )
}

#[test]
fn wait_recv_send() {
    let (s,r) = oneshot::<i32>();
    assert_eq!(
        block_on(
            zip!(async { let mut s = s.wait().await?; s.send(42) }, r)
        ),
        (Ok(()), Ok(42))
    )
}

#[test]
fn close_wait() {
    let (s,r) = oneshot::<bool>();
    r.close();
    assert_eq!(Closed(), block_on(s.wait()).unwrap_err());
}
