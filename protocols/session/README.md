# hopr-protocol-session

Contains implementation of Session protocol according to HOPR RFC-0008

Part of the [HOPR](https://hoprnet.org/) protocol implementation.

## Specification

- [RFC-0008: Session Protocol](https://github.com/hoprnet/rfc/blob/main/rfcs/RFC-0008-session-protocol/0008-session-protocol.md)

## Tracing context

Sockets retain their construction span for later IO polls and state workers. Construct a socket
under a session root span to attach a session identifier once; nested processing and acknowledgement
spans inherit it. `HoprSession` supplies this root in the HOPR transport.

## License

GPL-3.0-only
