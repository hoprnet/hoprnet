#![feature(test)]
#![allow(deprecated)]

use std::task::Poll;
use futures_lite::future::block_on;
use futures_micro::{poll_fn, prelude::*};

fn ready<T>(x: T) -> impl Future<Output = T> {
    let mut x = Some(x);
    poll_fn(move |_| Poll::Ready(x.take().unwrap()))
}

fn pending<T>() -> impl Future<Output = T> {
    poll_fn(|_| Poll::Pending)
}

#[test]
fn waker_sleep_test() {
    assert!(block_on(async {
        let waker = waker().await;
        waker.wake();
        sleep().await;
        true
    }));
}

#[test]
fn yield_once_test() {
    assert_eq!(
        true,
        block_on(async {
            yield_once().await;
            true
        })
    );
    assert_eq!(
        false,
        block_on(or!(
            async {
                yield_once().await;
                true
            },
            ready(false)
        ))
    );
}

#[test]
fn or_test() {
    assert_eq!(false, block_on(or(pending::<bool>(), ready(false))));
    assert_eq!(1, block_on(or!(ready(1), ready(2), ready(3))));
    assert_eq!(2, block_on(or!(pending(), ready(2), ready(3))));
    assert_eq!(3, block_on(or!(pending(), pending(), ready(3))));
}

#[test]
fn zip_test() {
    assert_eq!((true, false), block_on(zip(ready(true), ready(false))));
    assert_eq!((1, 2, 3), block_on(zip!(ready(1), ready(2), ready(3))));

    assert_eq!(
        (1, false, 3),
        block_on(zip!(ready(1), ready(false), ready(3)))
    );
}
