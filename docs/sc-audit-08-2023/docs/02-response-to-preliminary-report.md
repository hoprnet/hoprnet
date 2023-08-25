## Response to Preliminary Report

All relevant test files have been appropriately updated in alignment with the findings outlined below.

Updated Git commit hash under audit is:

```
e6ac2da904672da5c932b3e95ba6cbc37934643d
```

### Findings

#### 5.1 HoprChannels ERC777 Reentrancy

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- Modified `_closeIncomingChannelInternal` and `_finalizeOutgoingChannelClosureInternal` to set the channel balance to 0 before the reentrant call to `token.transfer`.
- In the mentioned functions and in `_redeemTicketInternal`, events are now emitted before calling `token.transfer`.
- Adjusted the order of event emission in the test case `test_redeemTicket_bidirectional`.

**Commit Hash:** [60f9facc23db19ef7ae93845086dc6bc6ee15101](https://github.com/hoprnet/hoprnet/commit/60f9facc23db19ef7ae93845086dc6bc6ee15101)

#### 5.2 Winning Ticket Can Be Redeemed Multiple Times

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- Modified calculation of `ticketIndex` to be based on `redeemable.data.ticketIndex` in `_redeemTicketInternal`.

**Commit Hash:** [2ca5c86d334950c0e40e06d5999f5996c5eccc37](https://github.com/hoprnet/hoprnet/commit/2ca5c86d334950c0e40e06d5999f5996c5eccc37)

#### 5.3 EIP-712 Incompliant Signed Message

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- Updated encoding from `abi.encode` to `abi.encodePacked` in specific functions within certain contracts (see details).

**Commit Hash:** [dd99732fed5c78b76de6f26d5b86579a00ef2964](https://github.com/hoprnet/hoprnet/commit/dd99732fed5c78b76de6f26d5b86579a00ef2964)

#### 5.4 Dependencies Between Source File Folders

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- Reorganized the placement of specific library files and contracts to align with improved structuring and imports.

**Commit Hash:** [8f09d13dc76d27031ece289690a6515fef598663](https://github.com/hoprnet/hoprnet/commit/8f09d13dc76d27031ece289690a6515fef598663)

#### 5.5 DomainSeparator Is Not Recomputed After a Change of Chain ID

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- Removed the `immutable` keyword from `domainSeparator` in "HoprNodeSafeRegistry" and "HoprChannels" contracts.
- Introduced publicly callable functions `updateDomainSeparator` and `updateLedgerDomainSeparator` to adjust domain separators in relevant contracts in case of a fork.
- Emitted an event when the domain separator gets updated to a different value.

**Commit Hash:** [d51ce5d2bec87a81004488327d7282b22f97f38b](https://github.com/hoprnet/hoprnet/commit/d51ce5d2bec87a81004488327d7282b22f97f38b)

#### 5.6 HoprNodeSafeRegistry Is Not an INodeSafeRegistry

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- Removed the imported but unused `INodeSafeRegistry` from the "HoprChannels" contract.
- Adjusted the import of "HoprNodeSafeRegistry" in "HoprSafeProxyForNetworkRegistry" to refer directly to the contract, removing the interface.
- Removed the "INodeSafeRegistry" contract entirely as it is no longer in use.

**Commit Hash:** [19ec033e9351a7110d0d9bc172fc62c07878729d](https://github.com/hoprnet/hoprnet/commit/19ec033e9351a7110d0d9bc172fc62c07878729d)

#### 5.7 HoprNodeStakeFactory Can Clone Any Module

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- Adjusted the `NewHoprNodeStakeModule` event to include an indexed parameter, improving event clarity.
- Separated events from "HoprNodeStakeFactory" into an abstract contract named "HoprNodeStakeFactoryEvents."

**Commit Hash:** [eb9ba9c2a2d1158938e05d71ef92912048412752](https://github.com/hoprnet/hoprnet/commit/eb9ba9c2a2d1158938e05d71ef92912048412752)

#### 5.8 IHoprNodeSafeRegistry Is a Contract and Not an Interface

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- As described in response to "5.6 HoprNodeSafeRegistry Is Not an INodeSafeRegistry," the contract "IHoprNodeSafeRegistry" has been removed.

**Commit Hash:** [19ec033e9351a7110d0d9bc172fc62c07878729d](https://github.com/hoprnet/hoprnet/commit/19ec033e9351a7110d0d9bc172fc62c07878729d)

#### 5.9 Incorrect Flag Position Upper Bound

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- Adjusted the condition for reverting the `TooManyCapabilities` error in the `getDefaultCapabilityPermissionAt` function within the `TargetUtils` library.

**Commit Hash:** [baf6506001c97eb463c6a13ab20f8c4ae55120d9](https://github.com/hoprnet/hoprnet/commit/baf6506001c97eb463c6a13ab20f8c4ae55120d9)

#### 5.10 Incorrect Specifications and Comments

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- Adjusted relevant comments in `checkMultisendTransaction` and `decodeFunctionSigsAndPermissions` functions.

**Commit Hash:** [be2fedf82566341278830edf98b66d46dde3b740](https://github.com/hoprnet/hoprnet/commit/be2fedf82566341278830edf98b66d46dde3b740)

#### 5.11 Missing Input Checks at tokensReceived

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- Moved `validateBalance` and `validateChannelParties` from external functions (`fundChannelSafe` and `fundChannel`) to the internal function `_fundChannelInternal`. This allows `tokensReceived` to perform checks on balance and channel parties.
- Moved `_fundChannelInternal` before `token.transferFrom` in `fundChannelSafe` and `fundChannel` functions.

**Commit Hash:** [7f7431902f01939fc0c7ca359451bff41cf1224e](https://github.com/hoprnet/hoprnet/commit/7f7431902f01939fc0c7ca359451bff41cf1224e)

#### 5.12 Signatures Can Be Replayed

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- Created `NodeSafeRecord` struct that holds `nodeSigNonce` next to `safeAddress` mapping in `HoprNodeSafeRegistry`. The `nodeSigNonce` gets bumped when a node-safe pair is added.

**Commit Hash:** [b01ef3f74d63f766a98041a3508a56a68bbfdae9](https://github.com/hoprnet/hoprnet/commit/b01ef3f74d63f766a98041a3508a56a68bbfdae9)

#### 5.13 TargetUtils Incorrect Iterator Bound

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- Changed the offset in the loop in the `decodeDefaultPermissions` function from 176 to 184.
- Updated the respective code comment.

**Commit Hash:** [593970edb0f87a777714010afacb94c4e43162e0](https://github.com/hoprnet/hoprnet/commit/593970edb0f87a777714010afacb94c4e43162e0)

#### 5.14 Timestamp Is Not Updated With Snapshot

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- Updated `latestRoot.timestamp` with `block.timestamp` in `createSnapshot`.

**Commit Hash:** [5b943aafde3164b236891518543d52fdd01f30d0](https://github.com/hoprnet/hoprnet/commit/5b943aafde3164b236891518543d52fdd01f30d0)

#### 5.15 isNodeSafeRegistered Returns True for Unregistered Pairs if safeAddress==0

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- In `isNodeSafeRegistered`, return false when the node is not registered.

**Commit Hash:** [15cd4edf9c02bc1515db2e8ab33cdbbfeff0b955](https://github.com/hoprnet/hoprnet/commit/15cd4edf9c02bc1515db2e8ab33cdbbfeff0b955)

### Informational

#### 6.1 ERC777 Reentrancy in fundChannel

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- Restricted `nodeChainKeyAddress` to be an EOA
- Added `NodeIsContract` error, which gets thrown when the node is a contract
- Updated tests

**Commit Hash:** [38c33e2a748e3ca2e4fbf456190cdacb272fcc13](https://github.com/hoprnet/hoprnet/commit/38c33e2a748e3ca2e4fbf456190cdacb272fcc13)

### Others

#### 1. Rename and separate event definitions

**Commit Hash:** [2d5893a1b51a3e5f21408c233e098f8cf33928de](https://github.com/hoprnet/hoprnet/commit/2d5893a1b51a3e5f21408c233e098f8cf33928de)

**Description of Changes:**

- Removed `HoprChannels.Balance amount` from `ChannelOpened` event.
- Renamed `closureInitiationTime` to `closureTime` in `OutgoingChannelClosureInitiated` event.
- Split event definition of “HoprNetworkRegistry” contract into an abstract contract `HoprNetworkRegistryEvents`. The abstract event contract is inherited by the “HoprNetworkRegistry” contract.
- Renamed `EnabledNetworkRegistry` event to `NetworkRegistryStatusUpdated`.
- Renamed error from `NotSafeOwnerNorNode` to `NodeNotModuleMember` in `NodeSafeRegistry.sol`.

#### 2. Add more event in Simplified

**Description of Changes:**

- Added `ExecutionSuccess` and `ExecutionFailure` events in `SimplifiedModuleEvents`, which is inherited by `SimplifiedModule`. This allows HOPR daemon to listen to update in transaction execution results from the node management module proxy in addition to safe proxy.

**Commit Hash:** [c647548f63380e80a3253439fb7b91150cea59fe](https://github.com/hoprnet/hoprnet/commit/c647548f63380e80a3253439fb7b91150cea59fe)

#### 3. Improve HoprChannels

- When funding two channels with `tokenReceived`, if the token balance is zero error `InvalidBalance` gets reverted.
- Always emit `ChannelBalanceIncreased` in a successful `tokenReceived` call.
- Use tight packing for `HoprChannels.tokensReceived()` payload, meaning 40 bytes for `fundChannel` and 64 bytes for `fundChannelMulti`
- Document bidirectional channel funding in /`ERC777.tokensReceived()` is not intended for production use.
- Use packet encoding for ticket hashing to align with off-chain ticket usage.

#### 4. Bump Safe to v1.4.1

- Change contract deployment bytecode in `packages/ethereum/contracts/test/utils/SafeSingleton.sol`
- Update deployed addresses in `packages/ethereum/contracts/src/utils/SafeSuiteLib.sol`

#### 5. Improve HoprNodeStakeFactory

- Enhance code comments
- Make imports explicit
- Make the encoded address of the contract's approver (`r`) immutable
- Remove duplicated `safeProxyAddr` definition in `clone()`

#### 6. Improve SimplifiedModule

- Extend comments to explain "avatar"

#### 7. Improve CapabilityPermissions

- Import `IERC20`, `IERC777` interfaces for definition of selector
- Make all the selector definition public

#### 8. Improve NodeSafeRegistry

- Unwrap struct `NodeSafe` into flattened `address safeAddress, address nodeChainKeyAddress`

#### 9. Format

- Format all the contracts
- Include HOPR logo
