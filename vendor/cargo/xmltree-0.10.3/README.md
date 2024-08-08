xmltree-rs
==========

[Documention](https://docs.rs/xmltree/)

A small library for parsing an XML file into an in-memory tree structure.

Not recommended for large XML files, as it will load the entire file into memory.

https://crates.io/crates/xmltree

## Usage

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
xmltree = "0.10"
```

### Feature-flags

* `attribute-order` - change the data structure that stores attributes into one that keeps a
consistent order. This changes the type definition and adds another dependency.

## Compatability with xml-rs
This crate will export some types from the xml-rs crate.  If your own crate also uses the xml-rs
crate, but with a different version, the types may be incompatible.  One way to solve this is to
only use the exported types, but sometimes that is not always possible.  In those cases you should
use a version of xmltree that matches the version of xml-rs you are using:

| xml-rs version | xmltree version |
|----------------|-----------------|
| 0.8            | 0.10            |
| 0.7            | 0.8             |
| 0.6            | 0.6             |


## Example

See the documentation for some examples:

https://docs.rs/xmltree/
