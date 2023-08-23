# array-init

[Documentation](https://docs.rs/array-init)

[Crates.io](https://crates.io/crates/array-init)


(still kinda experimental, API may change, may be secretly unsafe)

The `array-init` crate allows you to initialize arrays
with an initializer closure that will be called
once for each element until the array is filled.

This way you do not need to default-fill an array
before running initializers. Rust currently only
lets you either specify all initializers at once,
individually (`[a(), b(), c(), ...]`), or specify
one initializer for a `Copy` type (`[a(); N]`),
which will be called once with the result copied over.

# Examples:

```rust
# #![allow(unused)]
# extern crate array_init;

// Initialize an array of length 50 containing
// successive squares

let arr: [u32; 50] = array_init::array_init(|i| (i*i) as u32);

// Initialize an array from an iterator
// producing an array of [1,2,3,4] repeated

let four = [1u32,2,3,4];
let mut iter = four.iter().cloned().cycle();
let arr: [u32; 50] = array_init::from_iter(iter).unwrap();

// Closures can also mutate state. We guarantee that they will be called
// in order from lower to higher indices.

let mut last = 1u64;
let mut secondlast = 0;
let fibonacci: [u64; 50] = array_init::array_init(|_| {
    let this = last + secondlast;
    secondlast = last;
    last = this;
    this
});
```

Currently, using `from_iter` and `array_init` will incur additional
memcpys, which may be undesirable for a large array. This can be eliminated
by using the nightly feature of this crate, which uses unions to provide
panic-safety. Alternatively, if your array only contains `Copy` types,
you can use `array_init_copy` and `from_iter_copy`.

Sadly, cannot guarantee right now that any of these solutions will completely
eliminate a memcpy.
