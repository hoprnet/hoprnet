---
applyTo: "**/*.rs"
description: "Rust-specific guidelines"
---

- Prefer immutable structures.
- Use `Result` and `Option` types for error handling and optional values.
- Follow Rust's naming conventions: `snake_case` for variables and functions, `CamelCase` for types and traits.
- Leverage Rust's pattern matching capabilities.
- Write documentation comments using `///` for public items.
- Prefer async runtime agnostic code, when not possible, use `tokio` runtime utilities
- Use `async-trait` crate for async methods in traits.
- Prefix tracing macros with `tracing::` (e.g., `tracing::info!`).