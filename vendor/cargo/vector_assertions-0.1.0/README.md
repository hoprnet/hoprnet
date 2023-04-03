# Vector assertion

Compares if two vectors contain the same values.

## Example

```rust
#[macro_use] extern crate vector_assertions;
fn main() {
    let a = vec![1, 2];
    let b = vec![2, 1];
    assert_vec_eq!(a, b, "we are testing addition with {} and {}", "a", "b");
}
```
