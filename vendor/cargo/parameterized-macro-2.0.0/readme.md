### High level overview of test case generation

#### An example

With this macro we generate the following code (1) from the given parameterized test definition (0):

**parameterized test definition:**

```rust
// 0

#[parameterized(a = {1, 2}, b = { "wanderlust", "wanderer" })]
fn my_test(a: i32, b: &str) {
    assert!(a > 0 && b.starts_with("w"))
}
```


**generated test cases:**
```rust
// 1

#[cfg(test)]
mod my_test {
    #[test]
    fn case_0() {
        let a: i32 = 1;
        let b: &str = "wanderlust";
        assert!(a > 0 && b.starts_with("w"))
    }

    #[test]
    fn case_1() {
        let a: i32 = 2;
        let b: &str = "wanderer";
        assert!(a > 0 && b.starts_with("w"))
    }
}
```

More examples can be found in the `expand` crate, and the tests.

#### notes:
- The function name in (1) is the same as the module name in (0)

- Note that arguments are not limited to primitives; they can be any expression (assuming:)

- In a parameterized test case, the input arguments (which are expressions) specified in the attribute should evaluate
  to the same type as their identically named companions in the function signature.

- Tests executed from the workspace crate should be run individually, e.g.
    (`cargo test --package parameterized-macro --test tests individual_cases -- --exact`).
    Otherwise, if just `cargo test` is used, some generated test cases will run in an incorrect context setting.
