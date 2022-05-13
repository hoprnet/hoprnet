[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprNetworkRegistry

# Interface: HoprNetworkRegistry

## Hierarchy

- `BaseContract`

  ↳ **`HoprNetworkRegistry`**

## Table of contents

### Properties

- [\_deployedPromise](HoprNetworkRegistry.md#_deployedpromise)
- [\_runningEvents](HoprNetworkRegistry.md#_runningevents)
- [\_wrappedEmits](HoprNetworkRegistry.md#_wrappedemits)
- [address](HoprNetworkRegistry.md#address)
- [callStatic](HoprNetworkRegistry.md#callstatic)
- [contractName](HoprNetworkRegistry.md#contractname)
- [deployTransaction](HoprNetworkRegistry.md#deploytransaction)
- [estimateGas](HoprNetworkRegistry.md#estimategas)
- [filters](HoprNetworkRegistry.md#filters)
- [functions](HoprNetworkRegistry.md#functions)
- [interface](HoprNetworkRegistry.md#interface)
- [off](HoprNetworkRegistry.md#off)
- [on](HoprNetworkRegistry.md#on)
- [once](HoprNetworkRegistry.md#once)
- [populateTransaction](HoprNetworkRegistry.md#populatetransaction)
- [provider](HoprNetworkRegistry.md#provider)
- [removeListener](HoprNetworkRegistry.md#removelistener)
- [resolvedAddress](HoprNetworkRegistry.md#resolvedaddress)
- [signer](HoprNetworkRegistry.md#signer)

### Methods

- [\_checkRunningEvents](HoprNetworkRegistry.md#_checkrunningevents)
- [\_deployed](HoprNetworkRegistry.md#_deployed)
- [\_wrapEvent](HoprNetworkRegistry.md#_wrapevent)
- [accountToNodePeerId](HoprNetworkRegistry.md#accounttonodepeerid)
- [attach](HoprNetworkRegistry.md#attach)
- [connect](HoprNetworkRegistry.md#connect)
- [deployed](HoprNetworkRegistry.md#deployed)
- [disableRegistry](HoprNetworkRegistry.md#disableregistry)
- [emit](HoprNetworkRegistry.md#emit)
- [enableRegistry](HoprNetworkRegistry.md#enableregistry)
- [enabled](HoprNetworkRegistry.md#enabled)
- [fallback](HoprNetworkRegistry.md#fallback)
- [isAccountRegisteredAndEligible](HoprNetworkRegistry.md#isaccountregisteredandeligible)
- [isNodeRegisteredAndEligible](HoprNetworkRegistry.md#isnoderegisteredandeligible)
- [listenerCount](HoprNetworkRegistry.md#listenercount)
- [listeners](HoprNetworkRegistry.md#listeners)
- [nodePeerIdToAccount](HoprNetworkRegistry.md#nodepeeridtoaccount)
- [owner](HoprNetworkRegistry.md#owner)
- [ownerDeregister](HoprNetworkRegistry.md#ownerderegister)
- [ownerRegister](HoprNetworkRegistry.md#ownerregister)
- [queryFilter](HoprNetworkRegistry.md#queryfilter)
- [removeAllListeners](HoprNetworkRegistry.md#removealllisteners)
- [renounceOwnership](HoprNetworkRegistry.md#renounceownership)
- [requirementImplementation](HoprNetworkRegistry.md#requirementimplementation)
- [selfDeregister](HoprNetworkRegistry.md#selfderegister)
- [selfRegister](HoprNetworkRegistry.md#selfregister)
- [sync](HoprNetworkRegistry.md#sync)
- [transferOwnership](HoprNetworkRegistry.md#transferownership)
- [updateRequirementImplementation](HoprNetworkRegistry.md#updaterequirementimplementation)

## Properties

### \_deployedPromise

• **\_deployedPromise**: `Promise`<`Contract`\>

#### Inherited from

BaseContract.\_deployedPromise

#### Defined in

node_modules/ethers/node_modules/@ethersproject/contracts/lib/index.d.ts:98

___

### \_runningEvents

• **\_runningEvents**: `Object`

#### Index signature

▪ [eventTag: `string`]: `RunningEvent`

#### Inherited from

BaseContract.\_runningEvents

#### Defined in

node_modules/ethers/node_modules/@ethersproject/contracts/lib/index.d.ts:99

___

### \_wrappedEmits

• **\_wrappedEmits**: `Object`

#### Index signature

▪ [eventTag: `string`]: (...`args`: `any`[]) => `void`

#### Inherited from

BaseContract.\_wrappedEmits

#### Defined in

node_modules/ethers/node_modules/@ethersproject/contracts/lib/index.d.ts:102

___

### address

• `Readonly` **address**: `string`

#### Inherited from

BaseContract.address

#### Defined in

node_modules/ethers/node_modules/@ethersproject/contracts/lib/index.d.ts:77

___

### callStatic

• **callStatic**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `accountToNodePeerId` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `disableRegistry` | (`overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `enableRegistry` | (`overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `enabled` | (`overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `isAccountRegisteredAndEligible` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `isNodeRegisteredAndEligible` | (`hoprPeerId`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `nodePeerIdToAccount` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `ownerDeregister` | (`accounts`: `string`[], `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `ownerRegister` | (`accounts`: `string`[], `hoprPeerIds`: `string`[], `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `renounceOwnership` | (`overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `requirementImplementation` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `selfDeregister` | (`overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `selfRegister` | (`hoprPeerId`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `sync` | (`accounts`: `string`[], `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `updateRequirementImplementation` | (`_requirementImplementation`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |

#### Overrides

BaseContract.callStatic

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:410

___

### contractName

• **contractName**: ``"HoprNetworkRegistry"``

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:242

___

### deployTransaction

• `Readonly` **deployTransaction**: `TransactionResponse`

#### Inherited from

BaseContract.deployTransaction

#### Defined in

node_modules/ethers/node_modules/@ethersproject/contracts/lib/index.d.ts:97

___

### estimateGas

• **estimateGas**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `accountToNodePeerId` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `disableRegistry` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `enableRegistry` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `enabled` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isAccountRegisteredAndEligible` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isNodeRegisteredAndEligible` | (`hoprPeerId`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `nodePeerIdToAccount` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `ownerDeregister` | (`accounts`: `string`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `ownerRegister` | (`accounts`: `string`[], `hoprPeerIds`: `string`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `requirementImplementation` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `selfDeregister` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `selfRegister` | (`hoprPeerId`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `sync` | (`accounts`: `string`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `updateRequirementImplementation` | (`_requirementImplementation`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

BaseContract.estimateGas

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:536

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Deregistered` | (`account?`: `string`) => `DeregisteredEventFilter` |
| `Deregistered(address)` | (`account?`: `string`) => `DeregisteredEventFilter` |
| `DeregisteredByOwner` | (`account?`: `string`) => `DeregisteredByOwnerEventFilter` |
| `DeregisteredByOwner(address)` | (`account?`: `string`) => `DeregisteredByOwnerEventFilter` |
| `EligibilityUpdated` | (`account?`: `string`, `eligibility?`: `boolean`) => `EligibilityUpdatedEventFilter` |
| `EligibilityUpdated(address,bool)` | (`account?`: `string`, `eligibility?`: `boolean`) => `EligibilityUpdatedEventFilter` |
| `EnabledNetworkRegistry` | (`isEnabled?`: `boolean`) => `EnabledNetworkRegistryEventFilter` |
| `EnabledNetworkRegistry(bool)` | (`isEnabled?`: `boolean`) => `EnabledNetworkRegistryEventFilter` |
| `OwnershipTransferred` | (`previousOwner?`: `string`, `newOwner?`: `string`) => `OwnershipTransferredEventFilter` |
| `OwnershipTransferred(address,address)` | (`previousOwner?`: `string`, `newOwner?`: `string`) => `OwnershipTransferredEventFilter` |
| `Registered` | (`account?`: `string`, `hoprPeerId?`: ``null``) => `RegisteredEventFilter` |
| `Registered(address,string)` | (`account?`: `string`, `hoprPeerId?`: ``null``) => `RegisteredEventFilter` |
| `RegisteredByOwner` | (`account?`: `string`, `hoprPeerId?`: ``null``) => `RegisteredByOwnerEventFilter` |
| `RegisteredByOwner(address,string)` | (`account?`: `string`, `hoprPeerId?`: ``null``) => `RegisteredByOwnerEventFilter` |
| `RequirementUpdated` | (`requirementImplementation?`: `string`) => `RequirementUpdatedEventFilter` |
| `RequirementUpdated(address)` | (`requirementImplementation?`: `string`) => `RequirementUpdatedEventFilter` |

#### Overrides

BaseContract.filters

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:474

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `accountToNodePeerId` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `disableRegistry` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `enableRegistry` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `enabled` | (`overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `isAccountRegisteredAndEligible` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `isNodeRegisteredAndEligible` | (`hoprPeerId`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `nodePeerIdToAccount` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `ownerDeregister` | (`accounts`: `string`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `ownerRegister` | (`accounts`: `string`[], `hoprPeerIds`: `string`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `requirementImplementation` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `selfDeregister` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `selfRegister` | (`hoprPeerId`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `sync` | (`accounts`: `string`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `updateRequirementImplementation` | (`_requirementImplementation`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

BaseContract.functions

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:268

___

### interface

• **interface**: `HoprNetworkRegistryInterface`

#### Overrides

BaseContract.interface

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:247

___

### off

• **off**: `OnEvent`<[`HoprNetworkRegistry`](HoprNetworkRegistry.md)\>

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:263

___

### on

• **on**: `OnEvent`<[`HoprNetworkRegistry`](HoprNetworkRegistry.md)\>

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:264

___

### once

• **once**: `OnEvent`<[`HoprNetworkRegistry`](HoprNetworkRegistry.md)\>

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:265

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `accountToNodePeerId` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `disableRegistry` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `enableRegistry` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `enabled` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `isAccountRegisteredAndEligible` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `isNodeRegisteredAndEligible` | (`hoprPeerId`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `nodePeerIdToAccount` | (`arg0`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `owner` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `ownerDeregister` | (`accounts`: `string`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `ownerRegister` | (`accounts`: `string`[], `hoprPeerIds`: `string`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `renounceOwnership` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `requirementImplementation` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `selfDeregister` | (`overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `selfRegister` | (`hoprPeerId`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `sync` | (`accounts`: `string`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferOwnership` | (`newOwner`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `updateRequirementImplementation` | (`_requirementImplementation`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

BaseContract.populateTransaction

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:611

___

### provider

• `Readonly` **provider**: `Provider`

#### Inherited from

BaseContract.provider

#### Defined in

node_modules/ethers/node_modules/@ethersproject/contracts/lib/index.d.ts:80

___

### removeListener

• **removeListener**: `OnEvent`<[`HoprNetworkRegistry`](HoprNetworkRegistry.md)\>

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:266

___

### resolvedAddress

• `Readonly` **resolvedAddress**: `Promise`<`string`\>

#### Inherited from

BaseContract.resolvedAddress

#### Defined in

node_modules/ethers/node_modules/@ethersproject/contracts/lib/index.d.ts:96

___

### signer

• `Readonly` **signer**: `Signer`

#### Inherited from

BaseContract.signer

#### Defined in

node_modules/ethers/node_modules/@ethersproject/contracts/lib/index.d.ts:79

## Methods

### \_checkRunningEvents

▸ **_checkRunningEvents**(`runningEvent`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `runningEvent` | `RunningEvent` |

#### Returns

`void`

#### Inherited from

BaseContract.\_checkRunningEvents

#### Defined in

node_modules/ethers/node_modules/@ethersproject/contracts/lib/index.d.ts:119

___

### \_deployed

▸ **_deployed**(`blockTag?`): `Promise`<`Contract`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockTag?` | `BlockTag` |

#### Returns

`Promise`<`Contract`\>

#### Inherited from

BaseContract.\_deployed

#### Defined in

node_modules/ethers/node_modules/@ethersproject/contracts/lib/index.d.ts:112

___

### \_wrapEvent

▸ **_wrapEvent**(`runningEvent`, `log`, `listener`): `Event`

#### Parameters

| Name | Type |
| :------ | :------ |
| `runningEvent` | `RunningEvent` |
| `log` | `Log` |
| `listener` | `Listener` |

#### Returns

`Event`

#### Inherited from

BaseContract.\_wrapEvent

#### Defined in

node_modules/ethers/node_modules/@ethersproject/contracts/lib/index.d.ts:120

___

### accountToNodePeerId

▸ **accountToNodePeerId**(`arg0`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:343

___

### attach

▸ **attach**(`addressOrName`): [`HoprNetworkRegistry`](HoprNetworkRegistry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`HoprNetworkRegistry`](HoprNetworkRegistry.md)

#### Overrides

BaseContract.attach

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:244

___

### connect

▸ **connect**(`signerOrProvider`): [`HoprNetworkRegistry`](HoprNetworkRegistry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`HoprNetworkRegistry`](HoprNetworkRegistry.md)

#### Overrides

BaseContract.connect

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:243

___

### deployed

▸ **deployed**(): `Promise`<[`HoprNetworkRegistry`](HoprNetworkRegistry.md)\>

#### Returns

`Promise`<[`HoprNetworkRegistry`](HoprNetworkRegistry.md)\>

#### Overrides

BaseContract.deployed

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:245

___

### disableRegistry

▸ **disableRegistry**(`overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:345

___

### emit

▸ **emit**(`eventName`, ...`args`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` \| `EventFilter` |
| `...args` | `any`[] |

#### Returns

`boolean`

#### Inherited from

BaseContract.emit

#### Defined in

node_modules/ethers/node_modules/@ethersproject/contracts/lib/index.d.ts:125

___

### enableRegistry

▸ **enableRegistry**(`overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:349

___

### enabled

▸ **enabled**(`overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:353

___

### fallback

▸ **fallback**(`overrides?`): `Promise`<`TransactionResponse`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `TransactionRequest` |

#### Returns

`Promise`<`TransactionResponse`\>

#### Inherited from

BaseContract.fallback

#### Defined in

node_modules/ethers/node_modules/@ethersproject/contracts/lib/index.d.ts:113

___

### isAccountRegisteredAndEligible

▸ **isAccountRegisteredAndEligible**(`account`, `overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:355

___

### isNodeRegisteredAndEligible

▸ **isNodeRegisteredAndEligible**(`hoprPeerId`, `overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `hoprPeerId` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:360

___

### listenerCount

▸ **listenerCount**(`eventName?`): `number`

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` \| `EventFilter` |

#### Returns

`number`

#### Inherited from

BaseContract.listenerCount

#### Defined in

node_modules/ethers/node_modules/@ethersproject/contracts/lib/index.d.ts:126

___

### listeners

▸ **listeners**<`TEvent`\>(`eventFilter?`): `TypedListener`<`TEvent`\>[]

#### Type parameters

| Name | Type |
| :------ | :------ |
| `TEvent` | extends [`TypedEvent`](TypedEvent.md)<`any`, `any`, `TEvent`\> |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter?` | [`TypedEventFilter`](TypedEventFilter.md)<`TEvent`\> |

#### Returns

`TypedListener`<`TEvent`\>[]

#### Overrides

BaseContract.listeners

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:255

▸ **listeners**(`eventName?`): `Listener`[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

`Listener`[]

#### Overrides

BaseContract.listeners

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:258

___

### nodePeerIdToAccount

▸ **nodePeerIdToAccount**(`arg0`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:365

___

### owner

▸ **owner**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:367

___

### ownerDeregister

▸ **ownerDeregister**(`accounts`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `accounts` | `string`[] |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:369

___

### ownerRegister

▸ **ownerRegister**(`accounts`, `hoprPeerIds`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `accounts` | `string`[] |
| `hoprPeerIds` | `string`[] |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:374

___

### queryFilter

▸ **queryFilter**<`TEvent`\>(`event`, `fromBlockOrBlockhash?`, `toBlock?`): `Promise`<`TEvent`[]\>

#### Type parameters

| Name | Type |
| :------ | :------ |
| `TEvent` | extends [`TypedEvent`](TypedEvent.md)<`any`, `any`, `TEvent`\> |

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | [`TypedEventFilter`](TypedEventFilter.md)<`TEvent`\> |
| `fromBlockOrBlockhash?` | `string` \| `number` |
| `toBlock?` | `string` \| `number` |

#### Returns

`Promise`<`TEvent`[]\>

#### Overrides

BaseContract.queryFilter

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:249

___

### removeAllListeners

▸ **removeAllListeners**<`TEvent`\>(`eventFilter`): [`HoprNetworkRegistry`](HoprNetworkRegistry.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `TEvent` | extends [`TypedEvent`](TypedEvent.md)<`any`, `any`, `TEvent`\> |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](TypedEventFilter.md)<`TEvent`\> |

#### Returns

[`HoprNetworkRegistry`](HoprNetworkRegistry.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:259

▸ **removeAllListeners**(`eventName?`): [`HoprNetworkRegistry`](HoprNetworkRegistry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`HoprNetworkRegistry`](HoprNetworkRegistry.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:262

___

### renounceOwnership

▸ **renounceOwnership**(`overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:380

___

### requirementImplementation

▸ **requirementImplementation**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:384

___

### selfDeregister

▸ **selfDeregister**(`overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:386

___

### selfRegister

▸ **selfRegister**(`hoprPeerId`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `hoprPeerId` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:390

___

### sync

▸ **sync**(`accounts`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `accounts` | `string`[] |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:395

___

### transferOwnership

▸ **transferOwnership**(`newOwner`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `newOwner` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:400

___

### updateRequirementImplementation

▸ **updateRequirementImplementation**(`_requirementImplementation`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_requirementImplementation` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprNetworkRegistry.ts:405
