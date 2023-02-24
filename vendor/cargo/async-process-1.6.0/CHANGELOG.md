# Version 1.6.0

- Switch from `winapi` to `windows-sys` (#27)
- Remove the dependency on the `once_cell` crate to restore the MSRV (#26)
- Fix build failure with minimal-versions (#28)

# Version 1.5.0

- Implement `AsRawFd` for `ChildStd*` on Unix (#23)
- Implement I/O safety traits on Rust 1.63+ on Unix (#23)

# Version 1.4.0

- `Command::spawn` and `Command::output` no longer unconfigure stdio streams (#20)
- Implement `From<std::process::Command>` for `Command` (#21)

# Version 1.3.0

- Improve debug implementation of `Command` (#18)

# Version 1.2.0

- Implement `AsRawHandle` on `Child` on `Windows` (#17)

# Version 1.1.0

- Add `into_stdio` method to `ChildStdin`, `ChildStdout`, and `ChildStderr`. (#13)

# Version 1.0.2

- Use `kill_on_drop` only when the last reference to `ChildGuard` is dropped.

# Version 1.0.1

- Update `futures-lite`.

# Version 1.0.0

- Update dependencies and stabilize.

# Version 0.1.3

- Update dependencies.

# Version 0.1.2

- Add Unix and Windows extensions.
- Add `Command::reap_on_drop()` option.
- Add `Command::kill_on_drop()` option.

# Version 0.1.1

- Initial version
