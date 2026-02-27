# hopr-transport-p2p

This crate provides the libp2p-based transport layer used by HOPR.

## NAT traversal

When built with `runtime-tokio`, the transport enables:

- `identify` for address exchange
- `autonat` for reachability detection
- `relay` client and server behaviours
- `dcutr` for direct connection upgrade through relay-assisted hole punching
- `upnp` for local gateway port mapping

The swarm is configured to include relay transport support so relayed connectivity is available as a fallback path when direct dialing is not possible.

Own multiaddresses used for on-chain announcement are sourced from swarm-discovered addresses (`NewListenAddr` and confirmed external addresses), not directly from static config input.
