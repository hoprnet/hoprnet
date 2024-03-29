# Hopli

CLI tool to manage HOPR identity generation, decryption, funding and registering to network registry.

## Installation

hopli requires contract bindings (`../ethereum/contracts/bindings`) which is build by foundry (`forge bind`)

## Commands

### General arguments

#### Identity directory or path

Identities can be read from a directory or directly from a path.

When reading from a directory, identity files MUST contain "id" in their file name to be considered as an identity file.
An additional parameter which specifies its prefix can also be passed.

```
    --identity-directory "./test" \
    --identity-prefix node_ \
```

When reading from a path, use `--identity-from-path "./test/hopr.id"`

Path and directory can be passed at the same time. When both are provided, files from the directory are read first and file from path is read later.

Note: when CREATing identities, you must pass `--identity-directory`. `--identity-from-path` is not accepted

#### Password

Password can be passed either as an env variable `IDENTITY_PASSWORD`, or via a path to the password file `--password-path`, e.g. `--password-path ./.pwd`

#### Private key

Private key to signer wallet can be passed either as an env variable or as a command line argument `--private-key`, e.g. `--private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80`.

Unless specified, the name of the env variable is `PRIVATE_KEY` by default. In commands that accept two or more private keys, each private key has its own specific name. E.g. `MANAGER_PRIVATE_KEY` for network registry commands that are reserved for wallets with manager privilege.

### Create identities

To create identities, a [path to it](####Identity-directory-or-path) and a [password](####Password) must be provided.

```
hopli identity create \
  --identity-directory "./test" \
  --identity-prefix nodes_ \
  --number 2 \
  --password-path "./test/pwd"
```

#### Read identities

Read ethereum addresses from identities. A [path to it](####Identity-directory-or-path) and a [password](####Password) must be provided.

```
hopli identity read \
  --identity-directory "./test" \
  --identity-prefix node_ \
  --password-path "./test/pwd"
```

#### Update identities password

To update some identites' password. Two [passwords](####Password) must be provided:

- an old (and valid) password to the identity files. Either provided as cli argument `--password-path` or as an env variable `IDENTITY_PASSWORD`
- a new password to the identity files. Either provided as cli argument `--new-password-path` or as an env variable `NEW_IDENTITY_PASSWORD`

```
hopli identity update \
  --identity-directory "./test" \
  --identity-prefix node_ \
  --password-path "./test/pwd" \
  --new-password-path "./test/newpwd"
```

The identities will be modified inplace.

### Faucet

To fund nodes with password, a [path to it](####Identity-directory-or-path), a [password](####Password), and a [private key](####Private-key) to the faucet wallet (EOA) must be provided.

`--hopr-amount` and `native-amount` can be floating number

```
hopli faucet \
    --network anvil-localhost \
    --contracts-root "../ethereum/contracts" \
    --address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1,0xd057604a14982fe8d88c5fc25aac3267ea142a08 \
    --identity-directory "./test" --identity-prefix node_ \
    --password-path "./test/pwd" \
    --private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
    --hopr-amount 10 --native-amount 0.1
```

### Network registery

#### Register nodes

A manager (EOA) of the network registry can register node and safe pairs to the network. Nodes' Ethereum addresses can either be provided as string or read from identity files.

The private key to the manager wallet (EOA) should be provided as a cli argument as in [private key](####Private-key) or as an env variable `MANAGER_PRIVATE_KEY`,

Note that when registering a node, if the said node:

- has been registered with the given safe, skip it. (It's idempotent)
- has been registered to a different safe, remove the registration with the old safe and register with the new safe
- has not been registered to the network registry, register it.

After the registration, manager will also call "force-sync" to set all the added safes to be "eligible" to the network.

```
export MANAGER_PRIVATE_KEY=<bank_private_key> \
hopli network-registry manager-register \
    --network anvil-localhost \
    --contracts-root "../ethereum/contracts" \
    --node-address 0x9e820e68f8c024779ebcb6cd2edda1885e1dbe1f,0xb3724772badf4d8fffa186a5ca0bea87693a6c2a \
    --safe-address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1,0xd057604a14982fe8d88c5fc25aac3267ea142a08
```

with node identities in the network registry contract

```
hopli -- network-registry manager-register \
    --network anvil-localhost \
    --contracts-root "../ethereum/contracts" \
    --identity-directory "./test" --password-path "./test/pwd" \
    --node-address 0x9e820e68f8c024779ebcb6cd2edda1885e1dbe1f,0xb3724772badf4d8fffa186a5ca0bea87693a6c2a \
    --safe-address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1,0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1,0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1,0xd057604a14982fe8d88c5fc25aac3267ea142a08,0xd057604a14982fe8d88c5fc25aac3267ea142a08 \
    --private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

#### Deregister nodes

A manager (EOA) of the network registry can remove node and safe pairs from the network. Nodes' Ethereum addresses can either be provided as string or read from identity files.

The private key to the manager wallet (EOA) should be provided as a cli argument as in [private key](####Private-key) or as an env variable `MANAGER_PRIVATE_KEY`,

If the node address has not been registered in the network registry contract, it's will be skipped.

```
hopli -- network-registry manager-deregister \
    --network anvil-localhost \
    --contracts-root "../ethereum/contracts" \
    --node-address 0x9e820e68f8c024779ebcb6cd2edda1885e1dbe1f,0xb3724772badf4d8fffa186a5ca0bea87693a6c2a \
    --private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

#### Sync eligibility

A manager (EOA) of the network registry can forcely set eligibility of safes.

The private key to the manager wallet (EOA) should be provided as a cli argument as in [private key](####Private-key) or as an env variable `MANAGER_PRIVATE_KEY`,

```
hopli -- network-registry manager-force-sync \
    --network anvil-localhost \
    --contracts-root "../ethereum/contracts" \
    --node-address 0x9e820e68f8c024779ebcb6cd2edda1885e1dbe1f,0xb3724772badf4d8fffa186a5ca0bea87693a6c2a \
    --eligibility true \
    --private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

### Safe module

To launch a HOPR node, it requires a Safe with HOPR node management module to be pre-established.
Such a safe is designed to hold assets (wxHOPR tokens in particular) so that funds are held outside of node for better security.
HOPR node management module is a plugin to the safe so that permitted nodes can interact with the Safe to transfer funds between the said Safe and the HoprChannels smart contract.

The private key to the deployer wallet (EOA) should be provided as a cli argument `--private-key` as in [private key](####Private-key) or as an env variable `PRIVATE_KEY`.

The private key to the manager wallet (EOA) should be provided as a cli argument `--manager-private-key` as in [private key](####Private-key) or as an env variable `MANAGER_PRIVATE_KEY`.

#### Create: express create and setup a safe and a module

Express create a safe and a module instances, and

- set default permissions to scope channels and token contracts as targets.
- add announcement as a permitted target in the deployed module proxy
- approve token transfer to be done for the safe by channels contracts
- if node addresses are known, include nodes to the module by the safe
- set desired threshold
- if no `admin-address` is provided, the only admin will be the caller (wallet of the `private-key`)
- fund safe with wxHOPR tokens
- if node addresses are known, fund nodes with native tokens
- if node addresses are known and a manager private key is proivded, add created safe and node to the network registry.

```
hopli safe-module create \
    --network anvil-localhost \
    --contracts-root "../ethereum/contracts" \
    --identity-directory "./test" \
    --password-path "./test/pwd" \
    --admin-address 0x47f2710069F01672D01095cA252018eBf08bF85e,0x0D07Eb66Deb54D48D004765E13DcC028cf56592b \
    --allowance 10.5 \
    --hopr-amount 10 \
    --native-amount 0.1 \
    --manager-private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
    --private-key 59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
```

#### Migrate: make safe and module compatible with a new network

Migrate an exising set of node(d) with safe and module to a new network:

- add the Channel contract of the new network to the module as target and set default permissions.
- add the Announcement contract as target to the module
- approve HOPR tokens of the Safe proxy to be transferred by the new Channels contract
- Use the manager wallet to add nodes and Safes to the Network Registry contract of the new network.

```
hopli safe-module migrate \
    --network anvil-localhost2 \
    --contracts-root "../ethereum/contracts" \
    --identity-directory "./test" \
    --password-path "./test/pwd" \
    --safe-address 0x6a64fe01c3aba5bdcd04b81fef375369ca47326f \
    --module-address 0x5d46d0c5279fd85ce7365e4d668f415685922839 \
    --manager-private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
    --private-key 59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
```

#### Move: move registered nodes to a new pair of safe and module

For each node, if the node has been registered to the NodeSafeRegistry, deregister itself and register it to the new pair of safe and module.

- use old safes to deregister nodes from Node-safe registry
- use the new safe to include nodes to the module
- use manager wallet to deregister nodes from the network registry
- use manager wallet to register nodes with new safes to the network regsitry

```
hopli safe-module safe-module move \
    --network anvil-localhost \
    --contracts-root "../ethereum/contracts"  \
    --old-module-address 0x5d46d0c5279fd85ce7365e4d668f415685922839 \
    --new-safe-address 0xce66d19a86600f3c6eb61edd6c431ded5cc92b21 \
    --new-module-address 0x3086c20265cf742b169b05cd0eae1941455e4e9f \
    --node-address 0x93a50B0fFF7b4ED36A3C6445e280E72AC2AEFc51,0x58033D3074D001a32bF379801eaf8969817fFfCf,0xeEDaab91158928647a9270Fe290897eBB1230250 \
    --manager-private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
    --private-key 59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
```
