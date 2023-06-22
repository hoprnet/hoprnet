# version
`version` is a very simple library who's job is to return the version of your crate if you're building with Cargo.

## Usage:
```rust
#[macro_use]
extern crate version;

// ...

version!() // Returns something like "1.0.0"

let ver : Version = FromStr::from_str( version!() ).unwrap();
```

## Notes:
This only works if you're building with Cargo since the macro fetches the version digits from enviroment variables set by Cargo ( `CARGO_PKG_VERSION_{MAJOR, MINOR, PATCH}` ).
