# HOPR Ethereum Package

Draft readme, for rust migration

## Installation

1. `rustup`
2. `foundryup`

### Note

1. Three solc versions are needed

- 0.4: Permittable token, implementation of xHOPR. The permittable token implementation is extracted from the deployed xHOPR token. The only alternative done on the contract is to keep `pragma solidity` with the least version
- 0.6: Deployed Hoprtoken
- 0.8: More recent contracts

2. Dependencies are vendored directly into the repo. Some of them are locked to a specific version

```
forge install foundry-rs/forge-std \
openzeppelin-contracts=OpenZeppelin/openzeppelin-contracts@v4.4.2 \
openzeppelin-contracts-v3-0-1=OpenZeppelin/openzeppelin-contracts@v3.0.1 \
--no-git --no-commit
```

3. If `forge coverage` is not found in as a command, update `foundryup` to the [latest nightly release](https://github.com/foundry-rs/foundry/releases) may solve the issue.
