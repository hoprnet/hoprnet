# Ringbuffer
![Github Workflows](https://img.shields.io/github/actions/workflow/status/NULLx76/ringbuffer/rust.yml?style=for-the-badge)
[![Docs.rs](https://img.shields.io/badge/docs.rs-ringbuffer-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/ringbuffer)
[![Crates.io](https://img.shields.io/crates/v/ringbuffer?logo=rust&style=for-the-badge)](https://crates.io/crates/ringbuffer)

The ringbuffer crate provides safe fixed size circular buffers (ringbuffers) in rust.

Implementations for three kinds of ringbuffers, with a mostly similar API are provided:

| type                           | description                                                                                                                                                            |
|--------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [`AllocRingBuffer`][1]         | Ringbuffer allocated on the heap at runtime. This ringbuffer is still fixed size. This requires the alloc feature.                                                     |
| [`GrowableAllocRingBuffer`][2] | Ringbuffer allocated on the heap at runtime. This ringbuffer can grow in size, and is implemented as an `alloc::VecDeque` internally. This requires the alloc feature. |
| [`ConstGenericRingBuffer`][3]  | Ringbuffer which uses const generics to allocate on the stack.                                                                                                         |

All of these ringbuffers also implement the [RingBuffer][4] trait for their shared API surface.

[1]: https://docs.rs/ringbuffer/latest/ringbuffer/struct.AllocRingBuffer.html
[2]: https://docs.rs/ringbuffer/latest/ringbuffer/struct.GrowableAllocRingBuffer.html
[3]: https://docs.rs/ringbuffer/latest/ringbuffer/struct.ConstGenericRingBuffer.html
[4]: https://docs.rs/ringbuffer/latest/ringbuffer/trait.RingBuffer.html

MSRV: Rust 1.59

# Usage

```rust
use ringbuffer::{AllocRingBuffer, RingBuffer};

fn main() {
    let mut buffer = AllocRingBuffer::with_capacity(2);

    // First entry of the buffer is now 5.
    buffer.push(5);

    // The last item we pushed is 5
    assert_eq!(buffer.back(), Some(&5));

    // Second entry is now 42.
    buffer.push(42);
    assert_eq!(buffer.peek(), Some(&5));
    assert!(buffer.is_full());

    // Because capacity is reached the next push will be the first item of the buffer.
    buffer.push(1);
    assert_eq!(buffer.to_vec(), vec![42, 1]);
}

```

# Features

| name  | default | description                                                                                                  |
|-------|---------|--------------------------------------------------------------------------------------------------------------|
| alloc | ✓       | Disable this feature to remove the dependency on alloc. Disabling this feature  makes `ringbuffer` `no_std`. |

# License

Licensed under MIT License
