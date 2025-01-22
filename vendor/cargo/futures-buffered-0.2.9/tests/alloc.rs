use std::{
    alloc::{GlobalAlloc, System},
    future::ready,
    sync::atomic::AtomicUsize,
};

use futures::{
    stream::{self, FuturesUnordered},
    FutureExt, StreamExt,
};
use futures_buffered::{BufferedStreamExt, FuturesUnorderedBounded};

struct TrackingAllocator {
    alloc_count: AtomicUsize,
    dealloc_count: AtomicUsize,
    alloc: AtomicUsize,
    dealloc: AtomicUsize,
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        self.alloc
            .fetch_add(layout.size(), std::sync::atomic::Ordering::Relaxed);
        self.alloc_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        self.dealloc
            .fetch_add(layout.size(), std::sync::atomic::Ordering::Relaxed);
        self.dealloc_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        System.dealloc(ptr, layout)
    }
}

impl TrackingAllocator {
    fn reset(&self) {
        self.alloc_count
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.dealloc_count
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.alloc.store(0, std::sync::atomic::Ordering::Relaxed);
        self.dealloc.store(0, std::sync::atomic::Ordering::Relaxed);
    }
    fn report(&self) {
        let alloc_count = self.alloc_count.load(std::sync::atomic::Ordering::Relaxed);
        let dealloc_count = self
            .dealloc_count
            .load(std::sync::atomic::Ordering::Relaxed);
        let alloc = self.alloc.load(std::sync::atomic::Ordering::Relaxed);
        let dealloc = self.dealloc.load(std::sync::atomic::Ordering::Relaxed);
        println!("count1:   {:?}", alloc_count);
        println!("count2:   {:?}", dealloc_count);
        println!("alloc:    {:?}", alloc);
        println!("dealloc:  {:?}", dealloc);
        self.reset();
    }
}

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator {
    alloc_count: AtomicUsize::new(0),
    dealloc_count: AtomicUsize::new(0),
    alloc: AtomicUsize::new(0),
    dealloc: AtomicUsize::new(0),
};

#[cfg(not(miri))]
const BATCH: usize = 256;
#[cfg(not(miri))]
const TOTAL: usize = 512000;

#[cfg(miri)]
const BATCH: usize = 32;
#[cfg(miri)]
const TOTAL: usize = 128;

#[test]
fn futures_unordered() {
    ALLOCATOR.reset();

    let mut queue = FuturesUnordered::new();
    for i in 0..BATCH {
        queue.push(ready(i))
    }
    for i in BATCH..TOTAL {
        queue.next().now_or_never().unwrap();
        queue.push(ready(i))
    }
    for _ in 0..BATCH {
        queue.next().now_or_never().unwrap();
    }

    ALLOCATOR.report();
}

#[test]
fn futures_unordered_bounded() {
    ALLOCATOR.reset();

    let mut queue = FuturesUnorderedBounded::new(BATCH);
    for i in 0..BATCH {
        queue.push(ready(i))
    }
    for i in BATCH..TOTAL {
        queue.next().now_or_never().unwrap();
        queue.push(ready(i))
    }
    for _ in 0..BATCH {
        queue.next().now_or_never().unwrap();
    }
    drop(queue);

    ALLOCATOR.report();
}

#[test]
fn futures_unordered2() {
    ALLOCATOR.reset();

    let mut queue = futures_buffered::FuturesUnordered::new();
    for i in 0..BATCH {
        queue.push(ready(i))
    }
    for i in BATCH..TOTAL {
        queue.next().now_or_never().unwrap();
        queue.push(ready(i))
    }
    for _ in 0..BATCH {
        queue.next().now_or_never().unwrap();
    }
    drop(queue);

    ALLOCATOR.report();
}

#[test]
fn buffer_unordered() {
    ALLOCATOR.reset();

    let mut s = stream::iter((0..TOTAL).map(ready)).buffer_unordered(BATCH);
    while s.next().now_or_never().unwrap().is_some() {}

    ALLOCATOR.report();
}

#[test]
fn buffered_unordered() {
    ALLOCATOR.reset();

    let mut s = stream::iter((0..TOTAL).map(ready)).buffered_unordered(BATCH);
    while s.next().now_or_never().unwrap().is_some() {}

    ALLOCATOR.report();
}

#[test]
fn futures_join_all() {
    ALLOCATOR.reset();

    let _ = futures::future::join_all((0..BATCH).map(ready))
        .now_or_never()
        .unwrap();

    ALLOCATOR.report();
}

#[test]
fn buffered_join_all() {
    ALLOCATOR.reset();

    let _ = futures_buffered::join_all((0..BATCH).map(ready))
        .now_or_never()
        .unwrap();

    ALLOCATOR.report();
}
