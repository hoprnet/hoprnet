[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / AccessControl

# Class: AccessControl

## Hierarchy

- `BaseContract`

  ↳ **`AccessControl`**

## Table of contents

### Constructors

- [constructor](AccessControl.md#constructor)

### Properties

- [\_deployedPromise](AccessControl.md#_deployedpromise)
- [\_runningEvents](AccessControl.md#_runningevents)
- [\_wrappedEmits](AccessControl.md#_wrappedemits)
- [address](AccessControl.md#address)
- [callStatic](AccessControl.md#callstatic)
- [deployTransaction](AccessControl.md#deploytransaction)
- [estimateGas](AccessControl.md#estimategas)
- [filters](AccessControl.md#filters)
- [functions](AccessControl.md#functions)
- [interface](AccessControl.md#interface)
- [populateTransaction](AccessControl.md#populatetransaction)
- [provider](AccessControl.md#provider)
- [resolvedAddress](AccessControl.md#resolvedaddress)
- [signer](AccessControl.md#signer)

### Methods

- [DEFAULT\_ADMIN\_ROLE](AccessControl.md#default_admin_role)
- [\_checkRunningEvents](AccessControl.md#_checkrunningevents)
- [\_deployed](AccessControl.md#_deployed)
- [\_wrapEvent](AccessControl.md#_wrapevent)
- [attach](AccessControl.md#attach)
- [connect](AccessControl.md#connect)
- [deployed](AccessControl.md#deployed)
- [emit](AccessControl.md#emit)
- [fallback](AccessControl.md#fallback)
- [getRoleAdmin](AccessControl.md#getroleadmin)
- [getRoleMember](AccessControl.md#getrolemember)
- [getRoleMemberCount](AccessControl.md#getrolemembercount)
- [grantRole](AccessControl.md#grantrole)
- [hasRole](AccessControl.md#hasrole)
- [listenerCount](AccessControl.md#listenercount)
- [listeners](AccessControl.md#listeners)
- [off](AccessControl.md#off)
- [on](AccessControl.md#on)
- [once](AccessControl.md#once)
- [queryFilter](AccessControl.md#queryfilter)
- [removeAllListeners](AccessControl.md#removealllisteners)
- [removeListener](AccessControl.md#removelistener)
- [renounceRole](AccessControl.md#renouncerole)
- [revokeRole](AccessControl.md#revokerole)
- [getContractAddress](AccessControl.md#getcontractaddress)
- [getInterface](AccessControl.md#getinterface)
- [isIndexed](AccessControl.md#isindexed)

## Constructors

### constructor

• **new AccessControl**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |
| `contractInterface` | `ContractInterface` |
| `signerOrProvider?` | `Signer` \| `Provider` |

#### Inherited from

BaseContract.constructor

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:105

## Properties

### \_deployedPromise

• **\_deployedPromise**: `Promise`<`Contract`\>

#### Inherited from

BaseContract.\_deployedPromise

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:98

___

### \_runningEvents

• **\_runningEvents**: `Object`

#### Index signature

▪ [eventTag: `string`]: `RunningEvent`

#### Inherited from

BaseContract.\_runningEvents

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:99

___

### \_wrappedEmits

• **\_wrappedEmits**: `Object`

#### Index signature

▪ [eventTag: `string`]: (...`args`: `any`[]) => `void`

#### Inherited from

BaseContract.\_wrappedEmits

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:102

___

### address

• `Readonly` **address**: `string`

#### Inherited from

BaseContract.address

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:77

___

### callStatic

• **callStatic**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `DEFAULT_ADMIN_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `getRoleAdmin` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `getRoleMember` | (`role`: `BytesLike`, `index`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `getRoleMemberCount` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `grantRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `hasRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `renounceRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `revokeRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |

#### Overrides

BaseContract.callStatic

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:241

___

### deployTransaction

• `Readonly` **deployTransaction**: `TransactionResponse`

#### Inherited from

BaseContract.deployTransaction

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:97

___

### estimateGas

• **estimateGas**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `DEFAULT_ADMIN_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getRoleAdmin` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getRoleMember` | (`role`: `BytesLike`, `index`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getRoleMemberCount` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `grantRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `hasRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `renounceRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `revokeRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

BaseContract.estimateGas

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:338

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `RoleAdminChanged` | (`role?`: `BytesLike`, `previousAdminRole?`: `BytesLike`, `newAdminRole?`: `BytesLike`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`], { `newAdminRole`: `string` ; `previousAdminRole`: `string` ; `role`: `string`  }\> |
| `RoleAdminChanged(bytes32,bytes32,bytes32)` | (`role?`: `BytesLike`, `previousAdminRole?`: `BytesLike`, `newAdminRole?`: `BytesLike`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`], { `newAdminRole`: `string` ; `previousAdminRole`: `string` ; `role`: `string`  }\> |
| `RoleGranted` | (`role?`: `BytesLike`, `account?`: `string`, `sender?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`], { `account`: `string` ; `role`: `string` ; `sender`: `string`  }\> |
| `RoleGranted(bytes32,address,address)` | (`role?`: `BytesLike`, `account?`: `string`, `sender?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`], { `account`: `string` ; `role`: `string` ; `sender`: `string`  }\> |
| `RoleRevoked` | (`role?`: `BytesLike`, `account?`: `string`, `sender?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`], { `account`: `string` ; `role`: `string` ; `sender`: `string`  }\> |
| `RoleRevoked(bytes32,address,address)` | (`role?`: `BytesLike`, `account?`: `string`, `sender?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`], { `account`: `string` ; `role`: `string` ; `sender`: `string`  }\> |

#### Overrides

BaseContract.filters

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:282

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `DEFAULT_ADMIN_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getRoleAdmin` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getRoleMember` | (`role`: `BytesLike`, `index`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getRoleMemberCount` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `grantRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `hasRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `renounceRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `revokeRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

BaseContract.functions

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:161

___

### interface

• **interface**: `AccessControlInterface`

#### Overrides

BaseContract.interface

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:159

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `DEFAULT_ADMIN_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getRoleAdmin` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getRoleMember` | (`role`: `BytesLike`, `index`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getRoleMemberCount` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `grantRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `hasRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `renounceRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `revokeRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

BaseContract.populateTransaction

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:382

___

### provider

• `Readonly` **provider**: `Provider`

#### Inherited from

BaseContract.provider

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:80

___

### resolvedAddress

• `Readonly` **resolvedAddress**: `Promise`<`string`\>

#### Inherited from

BaseContract.resolvedAddress

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:96

___

### signer

• `Readonly` **signer**: `Signer`

#### Inherited from

BaseContract.signer

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:79

## Methods

### DEFAULT\_ADMIN\_ROLE

▸ **DEFAULT_ADMIN_ROLE**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:202

___

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

node_modules/@ethersproject/contracts/lib/index.d.ts:119

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

node_modules/@ethersproject/contracts/lib/index.d.ts:112

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

node_modules/@ethersproject/contracts/lib/index.d.ts:120

___

### attach

▸ **attach**(`addressOrName`): [`AccessControl`](AccessControl.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`AccessControl`](AccessControl.md)

#### Overrides

BaseContract.attach

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:120

___

### connect

▸ **connect**(`signerOrProvider`): [`AccessControl`](AccessControl.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`AccessControl`](AccessControl.md)

#### Overrides

BaseContract.connect

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:119

___

### deployed

▸ **deployed**(): `Promise`<[`AccessControl`](AccessControl.md)\>

#### Returns

`Promise`<[`AccessControl`](AccessControl.md)\>

#### Overrides

BaseContract.deployed

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:121

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

node_modules/@ethersproject/contracts/lib/index.d.ts:125

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

node_modules/@ethersproject/contracts/lib/index.d.ts:113

___

### getRoleAdmin

▸ **getRoleAdmin**(`role`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:204

___

### getRoleMember

▸ **getRoleMember**(`role`, `index`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | `BytesLike` |
| `index` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:206

___

### getRoleMemberCount

▸ **getRoleMemberCount**(`role`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:212

___

### grantRole

▸ **grantRole**(`role`, `account`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | `BytesLike` |
| `account` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:217

___

### hasRole

▸ **hasRole**(`role`, `account`, `overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | `BytesLike` |
| `account` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:223

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

node_modules/@ethersproject/contracts/lib/index.d.ts:126

___

### listeners

▸ **listeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter?`): `TypedListener`<`EventArgsArray`, `EventArgsObject`\>[]

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter?` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

`TypedListener`<`EventArgsArray`, `EventArgsObject`\>[]

#### Overrides

BaseContract.listeners

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:123

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

packages/ethereum/src/types/AccessControl.d.ts:146

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`AccessControl`](AccessControl.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | `TypedListener`<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`AccessControl`](AccessControl.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:126

▸ **off**(`eventName`, `listener`): [`AccessControl`](AccessControl.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`AccessControl`](AccessControl.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:147

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`AccessControl`](AccessControl.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | `TypedListener`<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`AccessControl`](AccessControl.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:130

▸ **on**(`eventName`, `listener`): [`AccessControl`](AccessControl.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`AccessControl`](AccessControl.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:148

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`AccessControl`](AccessControl.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | `TypedListener`<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`AccessControl`](AccessControl.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:134

▸ **once**(`eventName`, `listener`): [`AccessControl`](AccessControl.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`AccessControl`](AccessControl.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:149

___

### queryFilter

▸ **queryFilter**<`EventArgsArray`, `EventArgsObject`\>(`event`, `fromBlockOrBlockhash?`, `toBlock?`): `Promise`<[`TypedEvent`](../interfaces/TypedEvent.md)<`EventArgsArray` & `EventArgsObject`\>[]\>

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `fromBlockOrBlockhash?` | `string` \| `number` |
| `toBlock?` | `string` \| `number` |

#### Returns

`Promise`<[`TypedEvent`](../interfaces/TypedEvent.md)<`EventArgsArray` & `EventArgsObject`\>[]\>

#### Overrides

BaseContract.queryFilter

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:153

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`AccessControl`](AccessControl.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`AccessControl`](AccessControl.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:142

▸ **removeAllListeners**(`eventName?`): [`AccessControl`](AccessControl.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`AccessControl`](AccessControl.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:151

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`AccessControl`](AccessControl.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | `TypedListener`<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`AccessControl`](AccessControl.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:138

▸ **removeListener**(`eventName`, `listener`): [`AccessControl`](AccessControl.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`AccessControl`](AccessControl.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:150

___

### renounceRole

▸ **renounceRole**(`role`, `account`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | `BytesLike` |
| `account` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:229

___

### revokeRole

▸ **revokeRole**(`role`, `account`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | `BytesLike` |
| `account` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/AccessControl.d.ts:235

___

### getContractAddress

▸ `Static` **getContractAddress**(`transaction`): `string`

#### Parameters

| Name | Type |
| :------ | :------ |
| `transaction` | `Object` |
| `transaction.from` | `string` |
| `transaction.nonce` | `BigNumberish` |

#### Returns

`string`

#### Inherited from

BaseContract.getContractAddress

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:106

___

### getInterface

▸ `Static` **getInterface**(`contractInterface`): `Interface`

#### Parameters

| Name | Type |
| :------ | :------ |
| `contractInterface` | `ContractInterface` |

#### Returns

`Interface`

#### Inherited from

BaseContract.getInterface

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:110

___

### isIndexed

▸ `Static` **isIndexed**(`value`): value is Indexed

#### Parameters

| Name | Type |
| :------ | :------ |
| `value` | `any` |

#### Returns

value is Indexed

#### Inherited from

BaseContract.isIndexed

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:116
