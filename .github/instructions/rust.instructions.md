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
- Default to native async fn in traits (Rust 1.75+) for static dispatch—this avoids unnecessary heap allocations and is the idiomatic approach.
- Reserve async-trait only for dynamic dispatch scenarios where you need dyn Trait object compatibility or have MSRV constraints below 1.75.
- For public traits, note that explicit Send bounds should be handled (via trait-variant or spelling out return types as impl Future + Send), as the compiler warns about implicit auto-trait assumptions.
- Prefix tracing macros with `tracing::` (e.g., `tracing::info!`).
