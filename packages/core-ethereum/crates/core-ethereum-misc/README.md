# core-ethereum-misc

This crate contains various on-chain related modules and types:

- `chain`: create payloads for various transactions used throughout entire HOPR on-chain layer
- `constants`: constants related to on-chain operations
- `errors`: on-chain related error messages
- `network_registry`: implements the Network Registry check that's used for peer connection gating
- `redeem`: contains all ticket redemption logic
- `transaction_queue`: implements a queue of outgoing transactions that is executed one-by-one in the background
