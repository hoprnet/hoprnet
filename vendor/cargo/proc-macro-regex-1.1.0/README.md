# proc-macro-regex
A proc macro regex library to match an arbitrary string or byte array to a regular expression.
[![Build status](https://github.com/LinkTed/proc-macro-regex/workflows/Continuous%20Integration/badge.svg)](https://github.com/LinkTed/proc-macro-regex/actions?query=workflow%3A%22Continuous+Integration%22)
[![Latest version](https://img.shields.io/crates/v/proc-macro-regex.svg)](https://crates.io/crates/proc-macro-regex)
[![Dependency status](https://deps.rs/repo/github/linkted/proc-macro-regex/status.svg)](https://deps.rs/repo/github/linkted/proc-macro-regex)
[![License](https://img.shields.io/crates/l/proc-macro-regex.svg)](https://opensource.org/licenses/BSD-3-Clause)

## Usage
Add this to your `Cargo.toml`:
```toml
[dependencies]
proc-macro-regex = "~1.1.0"
```

## Example
The macro `regex!` creates a function of the given name which takes a string or byte array and 
returns `true` if the argument matches the regex, otherwise `false`.
```rust
use proc_macro_regex::regex;

/// Create the function with the signature:
/// fn regex_email(s: &str) -> bool; 
regex!(regex_email "^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\\.[a-zA-Z0-9-.]+$");

fn main () {
   println!("Returns true  == {}", regex_email("example@example.org"));
   println!("Returns false == {}", regex_email("example.example.org"));
}
```

The given regex works the same as in the [regex](https://crates.io/crates/regex) crate. If the `^` 
is at the beginning of the regex and `$` at the end then the whole string is checked, otherwise is 
check if the string contains the regex.

## How it works
The macro creates a *deterministic finite automaton* (DFA), which parse the given input. 
Depending on the size of the DFA or the character of the regex, a lookup table or a code base 
implementation (binary search) is generated. If the size of the lookup table would be bigger than 
65536 bytes (can be changed) then a code base implementation (binary search) is used. Additionally, 
if the regex contains any Unicode (no ASCII) character then a code base implementation 
(binary search) is used, too.

The following macro generates the following code:
```rust
regex!(example_1 "abc");
```
Generates:
```rust
fn example_1(s: &str) -> bool {
    static TABLE: [[u8; 256]; 3usize] = [ ... ];
    let mut state = 0;
    for c in s.bytes() {
        state = TABLE[state as usize][c as usize];
        if state == u8::MAX {
            return true;
        }
    }
    false
}
```

To tell the macro that the lookup table is not allowed to be bigger than 256 bytes, a third 
argument can be given. Therefore, a code base implementation (binary search) of the DFA is 
generated.
```rust
regex!(example_2 "abc" 256);
```
Generates:
```rust
fn example_2(s: &str) -> bool {
    let mut state = 0;
    for c in s.bytes() {
        state = if state < 1usize {
            match c {
                97u8 => 1usize,
                _ => 0usize,
            }
        } else {
            if state == 1usize {
                match c {
                    97u8 => 1usize,
                    98u8 => 2usize,
                    _ => 0usize,
                }
            } else {
                match c {
                    97u8 => 1usize,
                    99u8 => return true,
                    _ => 0usize,
                }
            }
        };
    }
    false
}
```

To change the visibility of the function, add the keywords at the beginning of the arguments. 
```rust
regex!(pub example_2 "abc" 256);
```
Generates:
```rust
pub fn example_3(s: &str) -> bool {
   // same as in example_1 (see above)
}
```

To parse a byte array instead of string, pass a byte string.
```rust
regex!(example_4 b"abc");
```
Generates:
```rust
fn example_4(s: &[u8]) -> bool {
   // same as in example_1 (see above)
}
```

The generated code should work with `#![no_std]`, too.

## proc-macro-regex vs regex
Advantages:
* Compile-time (no runtime initialization, no lazy-static)
* Generated code that does not contain any dependencies
* No heap allocation
* Approximately 12%-68% faster for no trivia regex [^1]

[^1]: It were tested with regex in `benches/compare.rs`. For pattern/word matching it is slower 
    because the [regex](https://crates.io/crates/regex) library uses 
    [aho-corasick](https://crates.io/crates/aho-corasick/). (See Performance)

Disadvantages:
* Currently, no group captures
* No runtime regex generation

### Performance
This is the performance comparison between this crate and the regex crate. If you want to test it 
by yourself, run `cargo bench --bench compare`.

| Name   | `proc-macro-regex` |      `regex` |  Result |
|--------|--------------:|-------------:|--------:|
| E-Mail |  743.95 MiB/s | 441.67 MiB/s | 68.44 % |
| URL    |  584.62 MiB/s | 519.00 MiB/s | 12.64 % |
| IPv6   |  746.92 MiB/s | 473.38 MiB/s | 57.78 % |

This was compiled with `rustc 1.53.0-nightly (392ba2ba1 2021-04-17)`.

## License
This project is licensed under the [BSD-3-Clause](https://opensource.org/licenses/BSD-3-Clause) 
license.

### Contribution
Any contribution intentionally submitted for inclusion in `proc-macro-regex` by you, shall 
be licensed as [BSD-3-Clause](https://opensource.org/licenses/BSD-3-Clause), without any additional 
terms or conditions.
