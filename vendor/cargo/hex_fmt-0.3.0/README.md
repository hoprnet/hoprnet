# Formatting and shortening byte slices as hexadecimal strings

This crate provides wrappers for byte slices and lists of byte slices that implement the
standard formatting traits and print the bytes as a hexadecimal string. It respects the
alignment, width and precision parameters and applies padding and shortening.

```rust
let bytes: &[u8] = &[0x0a, 0x1b, 0x2c, 0x3d, 0x4e, 0x5f];

assert_eq!("0a1b2c3d4e5f", &format!("{}", HexFmt(bytes)));

// By default the full slice is printed. Change the width to apply padding or shortening.
assert_eq!("0a..5f", &format!("{:6}", HexFmt(bytes)));
assert_eq!("0a1b2c3d4e5f", &format!("{:12}", HexFmt(bytes)));
assert_eq!("  0a1b2c3d4e5f  ", &format!("{:16}", HexFmt(bytes)));

// The default alignment is centered. Use `<` or `>` to align left or right.
assert_eq!("0a1b..", &format!("{:<6}", HexFmt(bytes)));
assert_eq!("0a1b2c3d4e5f    ", &format!("{:<16}", HexFmt(bytes)));
assert_eq!("..4e5f", &format!("{:>6}", HexFmt(bytes)));
assert_eq!("    0a1b2c3d4e5f", &format!("{:>16}", HexFmt(bytes)));

// Use e.g. `4.8` to set the minimum width to 4 and the maximum to 8.
assert_eq!(" 12 ", &format!("{:4.8}", HexFmt([0x12])));
assert_eq!("123456", &format!("{:4.8}", HexFmt([0x12, 0x34, 0x56])));
assert_eq!("123..89a", &format!("{:4.8}", HexFmt([0x12, 0x34, 0x56, 0x78, 0x9a])));

// If you prefer uppercase, use `X`.
assert_eq!("0A1B..4E5F", &format!("{:X}", HexFmt(bytes)));

// All of the above can be combined.
assert_eq!("0A1B2C..", &format!("{:<4.8X}", HexFmt(bytes)));

// With `HexList`, the parameters are applied to each entry.
let list = &[[0x0a; 3], [0x1b; 3], [0x2c; 3]];
assert_eq!("[0A.., 1B.., 2C..]", &format!("{:<4X}", HexList(list)));
```

