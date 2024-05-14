# Scalable Delayed Dealloc

[![Cargo](https://img.shields.io/crates/v/sdd)](https://crates.io/crates/sdd)
![Crates.io](https://img.shields.io/crates/l/sdd)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/wvwwvwwv/scalable-delayed-dealloc/sdd.yml?branch=main)

A scalable lock-free delayed memory reclaimer that deallocates virtual addresses only after it makes sure that there are no potential readers.

Its delayed deallocation algorithm is a variant of epoch-based reclamation where _retired_ memory chunks are stored in the thread-local storage until certain criteria are met. The [crossbeam_epoch](https://docs.rs/crossbeam-epoch/) create offers very similar functionality, however users will find this crate easier to use as the lifetime of a memory chunk is safely _managed_. For instance, `sdd::AtomicOwned` and `sdd::Owned` retire the contained instance when they are dropped, and `sdd::AtomicShared` and `sdd::Shared` retire the instance when the last strong reference is dropped.

## Memory Overhead

Retired instances are stored in intrusive queues in thread-local storage, and therefore additional 16-byte space for `Option<NonNull<dyn Collectible>>` is allocated per instance.

## Examples

This crate can be used _without an `unsafe` block_.

```rust
use sdd::{suspend, AtomicOwned, AtomicShared, Guard, Ptr, Shared, Tag};
use std::sync::atomic::Ordering::Relaxed;

// `atomic_shared` holds a strong reference to `17`.
let atomic_shared: AtomicShared<usize> = AtomicShared::new(17);

// `atomic_owned` owns `19`.
let atomic_owned: AtomicOwned<usize> = AtomicOwned::new(19);

// `guard` prevents the garbage collector from dropping reachable instances.
let guard = Guard::new();

// `ptr` cannot outlive `guard`.
let mut ptr: Ptr<usize> = atomic_shared.load(Relaxed, &guard);
assert_eq!(*ptr.as_ref().unwrap(), 17);

// `atomic_shared` can be tagged.
atomic_shared.update_tag_if(Tag::First, |p| p.tag() == Tag::None, Relaxed, Relaxed);

// `ptr` is not tagged, so CAS fails.
assert!(atomic_shared.compare_exchange(
    ptr,
    (Some(Shared::new(18)), Tag::First),
    Relaxed,
    Relaxed,
    &guard).is_err());

// `ptr` can be tagged.
ptr.set_tag(Tag::First);

// The ownership of the contained instance is transferred to the return value of CAS.
let prev: Shared<usize> = atomic_shared.compare_exchange(
    ptr,
    (Some(Shared::new(18)), Tag::Second),
    Relaxed,
    Relaxed,
    &guard).unwrap().0.unwrap();
assert_eq!(*prev, 17);

// `17` will be garbage-collected later.
drop(prev);

// `sdd::AtomicShared` can be converted into `sdd::Shared`.
let shared: Shared<usize> = atomic_shared.into_shared(Relaxed).unwrap();
assert_eq!(*shared, 18);

// `18` and `19` will be garbage-collected later.
drop(shared);
drop(atomic_owned);

// `17` is still valid as `guard` keeps the garbage collector from dropping it.
assert_eq!(*ptr.as_ref().unwrap(), 17);

// Execution of a closure can be deferred until all the current readers are gone.
guard.defer_execute(|| println!("deferred"));
drop(guard);

// If the thread is expected to lie dormant for a while, call `suspend()` to allow
// others to reclaim the memory.
suspend();
```

## Performance

- The average time taken to enter and exit a protected region: 2.0 nanoseconds on Apple M2.

## [Changelog](https://github.com/wvwwvwwv/scalable-delayed-dealloc/blob/main/CHANGELOG.md)
