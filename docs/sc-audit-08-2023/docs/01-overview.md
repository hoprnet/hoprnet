## Introduction

This documents provides an overview of the HOPR smart contracts as the basis for
the smart contracts audit in 08/2023. It describes the relevant threat model,
sets the scope, gives a high-level description of the relevant smart contracts
and their relation and ABIs. Moreover, it provides pointers to the source code
and steps on how to run tests and other operations on it.

## Scope

---
**FIXME:**

Pin final version prior to submitting for audit.

---

All HOPR smart contracts can be found in the hoprnet monorepo:

```
https://github.com/hoprnet/hoprnet
```

The Git commit hash under audit is:

```
d3dbcbe20975c1b0b4bd02fa8afaf42cf0d34c25
```

All smart contracts can be found within the folder:

```
packages/ethereum/contracts/src
```

For convenience, the following link points to the source folder using the
correct version:

```
https://github.com/hoprnet/hoprnet/tree/d3dbcbe20975c1b0b4bd02fa8afaf42cf0d34c25/packages/ethereum/contracts/src
```


Specifically, the following contracts are within the scope of the audit:

```bash
├── Channels.sol
├── Crypto.sol
├── MultiSig.sol
├── interfaces
│   ├── IAvatar.sol
│   ├── INetworkRegistryRequirement.sol
│   ├── INodeManagementModule.sol
│   └── INodeSafeRegistry.sol
├── node-stake
│   ├── NodeSafeRegistry.sol
│   ├── NodeStakeFactory.sol
│   └── permissioned-module
│       ├── CapabilityPermissions.sol
│       ├── NodeManagementModule.sol
│       └── SimplifiedModule.sol
└── utils
    ├── EnumerableStringSet.sol
    ├── EnumerableTargetSet.sol
    └── TargetUtils.sol
```

### Out of Scope

The following contracts are out of scope:

```bash
├── Announcements.sol # node announcement scheme which is independent from staking
├── NetworkRegistry.sol # implements network gate which will be removed eventually
├── proxy # implementations of adapters between network registry and staking
│   ├── DummyProxyForNetworkRegistry.sol
│   ├── SafeProxyForNetworkRegistry.sol
│   └── StakingProxyForNetworkRegistry.sol
└── static # existing contracts which will not be updated
    ├── EnumerableStringSet.sol
    ├── ERC777
    │   └── ERC777Snapshot.sol
    ├── HoprDistributor.sol
    ├── HoprForwarder.sol
    ├── HoprToken.sol
    ├── HoprWrapper.sol
    ├── HoprWrapperProxy.sol
    ├── openzeppelin-contracts
    │   ├── ERC777.sol
    │   └── README.md
    └── stake
        ├── HoprBoost.sol
        ├── HoprStake.sol
        ├── HoprStake2.sol
        ├── HoprStakeBase.sol
        ├── HoprStakeSeason3.sol
        ├── HoprStakeSeason4.sol
        ├── HoprStakeSeason5.sol
        ├── HoprStakeSeason6.sol
        ├── HoprStakeSeason7.sol
        ├── HoprWhitehat.sol
        └── IHoprBoost.sol
```

## Concepts

todo

### Staking

Presentation (WIP) which explains the new staking design:

```
https://docs.google.com/presentation/d/1oNG4LIBT0PKDHP1naOykdOaMaxlOxug7v3pQehwjBWc/edit?usp=sharing
```

todo

![alt](./sc-flow-1.png)
![alt](./sc-flow-2.png)
![alt](./sc-flow-3.png)
![alt](./sc-flow-4.png)
![alt](./sc-flow-5.png)

### Payment Channels

todo

## Threat Model

todo

## Contracts

The following is a short overview of each contract's purpose. Refer to the
contract's source code for more documentation.

### CapabilityPermissions.sol

### Channels.sol

### Crypto.sol

Bundles cryptographic primitives used by other contracts.

### EnumerableStringSet.sol

Adaptation of OpenZeppelin's EnumerableSet library for `string` type.

### EnumerableTargetSet.sol

Adaptation of OpenZeppelin's EnumerableSet and EnumerableMap (`AddressToUintMap`)
libraries for `TargetDefaultPermissions` type.

### IAvatar.sol

Interface for Avatar (Safe). Slightly enhanced version based on the original
from Safe.

### INodeManagementModule.sol

Interface for custom functions exposed by the `HoprNodeManagementModule`
contract.

### INodeSafeRegistry.sol

Minimum interface for `NodeSafeRegistry` contract.

### NodeManagementModule.sol

Permissioned capability-based Safe module for checking HOPR nodes operations.

### NodeSafeRegistry.sol

### NodeStakeFactory.sol

### SimplifiedModule.sol

### TargetUtils.sol

Helper functions for operations on `Target`s.

## Testing

All smart contracts in scope have test coverage using unit tests and fuzzy
tests. These tests use `forge` and may be executed by running the following
commands:

```bash
make deps
cd packages/ethereum/contracts
make sc-test
```

Coverage reports can be generated as well:

```bash
cd packages/ethereum/contracts
make sc-audit-coverage
firefox report/index.html
```
