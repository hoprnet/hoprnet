# Atomic Write File

[![Crate](https://img.shields.io/crates/v/atomic-write-file)](https://crates.io/crates/atomic-write-file) [![Documentation](https://img.shields.io/docsrs/atomic-write-file)](https://docs.rs/atomic-write-file/latest/atomic_write_file/) [![License](https://img.shields.io/crates/l/atomic-write-file)](https://choosealicense.com/licenses/bsd-3-clause/)

This is a Rust crate that offers functionality to write and overwrite files
*atomically*, that is: without leaving the file in an intermediate state.
Either the new contents of the files are written to the filesystem, or the old
contents (if any) are preserved.

This crate implements two main structs: `AtomicWriteFile` and `OpenOptions`,
which mimic the standard `std::fs::File` and `std::fs::OpenOptions` as much as
possible.

This crate supports all major platforms, including: Unix systems, Windows, and
WASI.

## Motivation and Example

Consider the following snippet of code to write a configuration file in JSON
format:

```rust
use std::io::Write;
use std::fs::File;

let mut file = File::options()
                    .write(true)
                    .create(true)
                    .open("config.json")?;

writeln!(file, "{{")?;
writeln!(file, "  \"key1\": \"value1\",")?;
writeln!(file, "  \"key2\": \"value2\"")?;
writeln!(file, "}}")?;
```

This code opens a file named `config.json`, truncates its contents (if the file
already existed), and writes the JSON content line-by-line.

If the code is interrupted before all of the `writeln!` calls are completed
(because of a panic, or a signal is received, or the process is killed, or a
filesystem error occurs), then the file will be left in a broken state: it will
not contain valid JSON data, and the original contents (if any) will be lost.

`AtomicWriteFile` solves this problem by placing the new contents into the
destination file only after it has been completely written to the filesystem.
The snippet above can be rewritten using `AtomicWriteFile` instead of `File` as
follows:

```rust
use std::io::Write;
use atomic_write_file::AtomicWriteFile;

let mut file = AtomicWriteFile::options()
                               .open("config.json")?;

writeln!(file, "{{")?;
writeln!(file, "  \"key1\": \"value1\",")?;
writeln!(file, "  \"key2\": \"value2\"")?;
writeln!(file, "}}")?;

file.commit()?;
```

Note that this code is almost the same as the original, except that it now uses
`AtomicWriteFile` instead of `File` and there's an additional call to
`commit()`.

If the code is interrupted early, before the call to `commit()`, the original
file `config.json` will be left untouched. Only if the new contents are fully
written to the filesystem, `config.json` will get them.

## Documentation

Reference, examples, internal details, and limitations are all available on
[docs.rs/atomic-write-file](https://docs.rs/atomic-write-file).
