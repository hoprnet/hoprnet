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

Private key to signer wallet can be passed either as an env variable `PRIVATE_KEY`, or as a command line argument `--private-key`, e.g. `--private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80`

### Create/Read identities
To create or read identities, a [path to it](####Identity-directory-or-path) and a [password](####Password) must be provided.
```
hopli identity \
  --action create \
  --identity-directory "./test" \
  --identity-prefix node_ \
  --number 3 \
  --password-path "./test/pwd"
```

Read ethereum addresses from identities

```
hopli identity \
  --action read \
  --identity-directory "./test" \
  --identity-prefix node_ \
  --password-path "./test/pwd"
```

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

The private key to the manager wallet (EOA) should be provided as in [private key](####Private-key).

Note that when registering a node, if the said node:
- has been registered with the given safe, skip it. (It's idempotent) 
- has been registered to a different safe, remove the registration with the old safe and register with the new safe
- has not been registered to the network registry, register it.

After the registration, manager will also call "force-sync" to set all the added safes to be "eligible" to the network.

```
export PRIVATE_KEY=<bank_private_key> \
hopli network-registry \
    --action manager-register \
    --network anvil-localhost \
    --contracts-root "../ethereum/contracts" \
    --node-address 0x9e820e68f8c024779ebcb6cd2edda1885e1dbe1f,0xb3724772badf4d8fffa186a5ca0bea87693a6c2a \
    --safe-address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1,0xd057604a14982fe8d88c5fc25aac3267ea142a08 \
    --private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 
```

with node identities in the network registry contract

```
hopli -- network-registry \
    --action manager-register \
    --network anvil-localhost \
    --contracts-root "../ethereum/contracts" \
    --identity-directory "./test" --password-path "./test/pwd" \
    --node-address 0x9e820e68f8c024779ebcb6cd2edda1885e1dbe1f,0xb3724772badf4d8fffa186a5ca0bea87693a6c2a \
    --safe-address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1,0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1,0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1,0xd057604a14982fe8d88c5fc25aac3267ea142a08,0xd057604a14982fe8d88c5fc25aac3267ea142a08 \
    --private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 
```

#### Deregister nodes
A manager (EOA) of the network registry can remove node and safe pairs from the network. Nodes' Ethereum addresses can either be provided as string or read from identity files.

The private key to the manager wallet (EOA) should be provided as in [private key](####Private-key).

If the node address has not been registered in the network registry contract, it's will be skipped. 

```
hopli -- network-registry \
    --action manager-deregister \
    --network anvil-localhost \
    --contracts-root "../ethereum/contracts" \
    --node-address 0x9e820e68f8c024779ebcb6cd2edda1885e1dbe1f,0xb3724772badf4d8fffa186a5ca0bea87693a6c2a \
    --private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 
```

#### Sync eligibility
A manager (EOA) of the network registry can forcely set eligibility of safes.

```
hopli -- network-registry \
    --action manager-force-sync \
    --network anvil-localhost \
    --contracts-root "../ethereum/contracts" \
    --node-address 0x9e820e68f8c024779ebcb6cd2edda1885e1dbe1f,0xb3724772badf4d8fffa186a5ca0bea87693a6c2a \
    --eligibility true \
    --private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 
```

Express create a safe and a module instances, then set default permissions

```
PRIVATE_KEY=<bank_private_key> \
hopli create-safe-module --network anvil-localhost \
    --identity-directory "./test" \
    --password-path "/test/.pwd" \
    --hopr-amount 10 --native-amount 0.1 \
    --contracts-root "../ethereum/contracts"
```

Migrate an exising set of node(d) with safe and module to a new network

```
PRIVATE_KEY=<safe_owner_private_key> DEPLOYER_PRIVATE_KEY=<network_registry_manager_key> \
hopli migrate-safe-module --network anvil-localhost \
    --identity-directory "./test" \
    --password-path "./test/.pwd" \
    --safe-address <safe_address> \
    --module-address <module_address> \
    --contracts-root "../ethereum/contracts"
```

Move a registered node to a new pair of safe and module

```
PRIVATE_KEY=<safe_owner_private_key> DEPLOYER_PRIVATE_KEY=<network_registry_manager_key> \
hopli move-node-to-safe-module --network anvil-localhost \
    --identity-directory "./test" \
    --password-path "./test/.pwd" \
    --safe-address <safe_address> \
    --module-address <module_address> \
    --contracts-root "../ethereum/contracts"
```

Sync or Force sync eligibility on Network Registry. Provide a comma-separated string of safe adresses in `safe-addresses`.
If `sync-type` sets to `normal-sync`, it will update the eligibility according to the actual eligibility of the staking account

```
PRIVATE_KEY=<network_registry_manager_key> DEPLOYER_PRIVATE_KEY=<network_registry_manager_key> \
hopli sync-network-registry --network anvil-localhost \
    --contracts-root "../ethereum/contracts" \
    --safe-addresses 0x4AAf51e0b43d8459AF85E33eEf3Ffb7EACb5532C,0x7d852faebb35adaed925869e028d9325bdd555a4,0xff7570ba5fc8bac26d4536565c48474e09f37b0d \
    --sync-type forced-sync \
    --eligibility true
```

## Development

### Run local development

```
cargo run -- -h
```

### Commands

Create 3 identity files in `./test` folder where password is saved in `.pwd` file

```
cargo run -- identity \
    --action create \
    --password-path ./.pwd \
    --identity-directory "./test" \
    --identity-prefix node_ \
    --number 3
```

Create 2 identity files in `./test` folder where password is stored as an environment variable `IDENTITY_PASSWORD`

```
IDENTITY_PASSWORD=switzerland \
cargo run -- identity \
    --action create \
    --identity-directory "./test" \
    --identity-prefix node_ \
    --number 2
```

Read ethereum addresses from identities

```
IDENTITY_PASSWORD=switzerland \
cargo run -- identity \
    --action read \
    --identity-directory "./test" \
    --identity-prefix node_

```

Fund nodes with password as env variable. Alternatively, a path to the password file can be provided with `--password-path`, e.g. `--password-path ./.pwd`

```
PRIVATE_KEY=<bank_private_key> \
IDENTITY_PASSWORD=local \
    cargo run -- faucet --network anvil-localhost \
    --use-local-identities --identity-directory "/tmp" \
    --address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1,0xd057604a14982fe8d88c5fc25aac3267ea142a08 \
    --contracts-root "../ethereum/contracts"  \
    --hopr-amount 10 --native-amount 0.1
```

Register some peer ids in the network registry contract

```
PRIVATE_KEY=<bank_private_key> \
    cargo run -- register-in-network-registry --network anvil-localhost \
    --node-address 16Uiu2HAmC9CRFeuF2cTf6955ECFmgDw6d27jLows7bftMqat5Woz,16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12 \
    --safe-address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1,0xd057604a14982fe8d88c5fc25aac3267ea142a08 \
    --contracts-root "../ethereum/contracts"
```

Register some peer ids as well as some node identities in the network registry contract

```
PRIVATE_KEY=<bank_private_key> \
IDENTITY_PASSWORD=local \
    cargo run -- register-in-network-registry --network anvil-localhost \
    --use-local-identities --identity-directory "/tmp" \
    --node-address 16Uiu2HAmC9CRFeuF2cTf6955ECFmgDw6d27jLows7bftMqat5Woz,16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12 \
    --safe-address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1,0xd057604a14982fe8d88c5fc25aac3267ea142a08 \
    --contracts-root "../ethereum/contracts"
```

> If foundry returns error that contains "HoprNetworkRegistry: Registry is disabled", run `cast send $(jq '.networks."anvil-localhost".network_registry_contract_address' ../ethereum/contracts/contracts-addresses.json) 'enableRegistry()' --rpc-url localhost:8545 --private-key $PRIVATE_KEY`

Express create a safe and a module instances, then set default permissions

```
PRIVATE_KEY=<bank_private_key> \
    cargo run -- create-safe-module --network anvil-localhost \
    --identity-directory "./test" \
    --password-path "./test/.pwd" \
    --hopr-amount 10 --native-amount 0.1 \
    --contracts-root "../ethereum/contracts"
```

Migrate an exising set of node(d) with safe and module to a new network

```
PRIVATE_KEY=<safe_owner_private_key> DEPLOYER_PRIVATE_KEY=<network_registry_manager_key> \
    cargo run -- migrate-safe-module --network anvil-localhost \
    --identity-directory "./test" \
    --password-path "./test/.pwd" \
    --safe-address <safe_address> \
    --module-address <module_address> \
    --contracts-root "../ethereum/contracts"
```

Move a registered node to a new pair of safe and module

```
PRIVATE_KEY=<safe_owner_private_key> DEPLOYER_PRIVATE_KEY=<network_registry_manager_key> \
    cargo run -- move-node-to-safe-module --network anvil-localhost \
    --identity-directory "./test" \
    --password-path "./test/.pwd" \
    --safe-address <safe_address> \
    --module-address <module_address> \
    --contracts-root "../ethereum/contracts"
```

Sync or Force sync eligibility on Network Registry. Provide a comma-separated string of safe adresses in `safe-addresses`.
If `sync-type` sets to `normal-sync`, it will update the eligibility according to the actual eligibility of the staking account

```
PRIVATE_KEY=<network_registry_manager_key> DEPLOYER_PRIVATE_KEY=<network_registry_manager_key> \
    cargo run -- sync-network-registry --network anvil-localhost \
    --contracts-root "../ethereum/contracts" \
    --safe-addresses 0x4AAf51e0b43d8459AF85E33eEf3Ffb7EACb5532C,0x7d852faebb35adaed925869e028d9325bdd555a4,0xff7570ba5fc8bac26d4536565c48474e09f37b0d \
    --sync-type forced-sync \
    --eligibility true
```

### Test

```
cargo test -- --nocapture
```

## Note:

1. When ` --use-local-identities`, the identity file should contain "id" in its name, either as part of the extention, or in the file stem.
