## ByteSize
[![Build Status](https://travis-ci.org/hyunsik/bytesize.svg?branch=master)](https://travis-ci.org/hyunsik/bytesize)
[![Crates.io Version](https://img.shields.io/crates/v/bytesize.svg)](https://crates.io/crates/bytesize)


ByteSize is an utility for human-readable byte count representation.

Features:
* Pre-defined constants for various size units (e.g., B, Kb, kib, Mb, Mib, Gb, Gib, ... PB)
* `ByteSize` type which presents size units convertible to different size units.
* Artimetic operations for `ByteSize`
* FromStr impl for `ByteSize`, allowing to parse from string size representations like 1.5KiB and 521TiB.
* Serde support for binary and human-readable deserializers like JSON

[API Documentation](https://docs.rs/bytesize/)

## Usage

Add this to your Cargo.toml:

```toml
[dependencies]
bytesize = {version = "1.2.0", features = ["serde"]}
```

and this to your crate root:
```rust
extern crate bytesize;
```

## Example
### Human readable representations (SI unit and Binary unit)
```rust
#[allow(dead_code)]
fn assert_display(expected: &str, b: ByteSize) {
  assert_eq!(expected, format!("{}", b));
}

#[test]
  fn test_display() {
    assert_display("215 B", ByteSize(215));
    assert_display("215 B", ByteSize::b(215));
    assert_display("1.0 KB", ByteSize::kb(1));
    assert_display("301.0 KB", ByteSize::kb(301));
    assert_display("419.0 MB", ByteSize::mb(419));
    assert_display("518.0 GB", ByteSize::gb(518));
    assert_display("815.0 TB", ByteSize::tb(815));
    assert_display("609.0 PB", ByteSize::pb(609));
  }

  fn assert_to_string(expected: &str, b: ByteSize, si: bool) {
    assert_eq!(expected.to_string(), b.to_string_as(si));
  }

  #[test]
  fn test_to_string() {
    assert_to_string("215 B", ByteSize(215), true);
    assert_to_string("215 B", ByteSize(215), false);

    assert_to_string("215 B", ByteSize::b(215), true);
    assert_to_string("215 B", ByteSize::b(215), false);

    assert_to_string("1.0 kiB", ByteSize::kib(1), true);
    assert_to_string("1.0 KB", ByteSize::kib(1), false);

    assert_to_string("293.9 kiB", ByteSize::kb(301), true);
    assert_to_string("301.0 KB", ByteSize::kb(301), false);

    assert_to_string("1.0 MiB", ByteSize::mib(1), true);
    assert_to_string("1048.6 KB", ByteSize::mib(1), false);

    assert_to_string("399.6 MiB", ByteSize::mb(419), true);
    assert_to_string("419.0 MB", ByteSize::mb(419), false);

    assert_to_string("482.4 GiB", ByteSize::gb(518), true);
    assert_to_string("518.0 GB", ByteSize::gb(518), false);

    assert_to_string("741.2 TiB", ByteSize::tb(815), true);
    assert_to_string("815.0 TB", ByteSize::tb(815), false);

    assert_to_string("540.9 PiB", ByteSize::pb(609), true);
    assert_to_string("609.0 PB", ByteSize::pb(609), false);
  }

  #[test]
  fn test_parsing_from_str() {
      // shortcut for writing test cases
      fn parse(s: &str) -> u64 {
          s.parse::<ByteSize>().unwrap().0
      }

      assert_eq!("0".parse::<ByteSize>().unwrap().0, 0);
      assert_eq!(parse("0"), 0);
      assert_eq!(parse("500"), 500);
      assert_eq!(parse("1K"), Unit::KiloByte * 1);
      assert_eq!(parse("1Ki"), Unit::KibiByte * 1);
      assert_eq!(parse("1.5Ki"), (1.5 * Unit::KibiByte) as u64);
      assert_eq!(parse("1KiB"), 1 * Unit::KibiByte);
      assert_eq!(parse("1.5KiB"), (1.5 * Unit::KibiByte) as u64);
      assert_eq!(parse("3 MB"), Unit::MegaByte * 3);
      assert_eq!(parse("4 MiB"), Unit::MebiByte * 4);
      assert_eq!(parse("6 GB"), 6 * Unit::GigaByte);
      assert_eq!(parse("4 GiB"), 4 * Unit::GibiByte);
      assert_eq!(parse("88TB"), 88 * Unit::TeraByte);
      assert_eq!(parse("521TiB"), 521 * Unit::TebiByte);
      assert_eq!(parse("8 PB"), 8 * Unit::PetaByte);
      assert_eq!(parse("8P"), 8 * Unit::PetaByte);
      assert_eq!(parse("12 PiB"), 12 * Unit::PebiByte);
  }
```

### Arithmetic operations
```rust
extern crate bytesize;

use bytesize::ByteSize;

fn byte_arithmetic_operator() {
  let x = ByteSize::mb(1);
  let y = ByteSize::kb(100);

  let plus = x + y;
  print!("{}", plus);

  let minus = ByteSize::tb(100) + ByteSize::gb(4);
  print!("{}", minus);
}
```
