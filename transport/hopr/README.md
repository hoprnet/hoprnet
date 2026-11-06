# hopr-transport

Implements the main public interface for the core HOPR library - that is the `Hopr` type.

The application representing the entire physical transport functionality, including:

- p2p
- protocol
- session

## Session tracing

See [the session tracing decision](docs/decisions/session-tracing.md) for span propagation,
logging configuration, and sender compatibility.
