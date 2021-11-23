[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprToken

# Class: HoprToken

## Hierarchy

- `BaseContract`

  ↳ **`HoprToken`**

## Table of contents

### Constructors

- [constructor](HoprToken.md#constructor)

### Properties

- [\_deployedPromise](HoprToken.md#_deployedpromise)
- [\_runningEvents](HoprToken.md#_runningevents)
- [\_wrappedEmits](HoprToken.md#_wrappedemits)
- [address](HoprToken.md#address)
- [callStatic](HoprToken.md#callstatic)
- [deployTransaction](HoprToken.md#deploytransaction)
- [estimateGas](HoprToken.md#estimategas)
- [filters](HoprToken.md#filters)
- [functions](HoprToken.md#functions)
- [interface](HoprToken.md#interface)
- [populateTransaction](HoprToken.md#populatetransaction)
- [provider](HoprToken.md#provider)
- [resolvedAddress](HoprToken.md#resolvedaddress)
- [signer](HoprToken.md#signer)

### Methods

- [DEFAULT\_ADMIN\_ROLE](HoprToken.md#default_admin_role)
- [MINTER\_ROLE](HoprToken.md#minter_role)
- [\_checkRunningEvents](HoprToken.md#_checkrunningevents)
- [\_deployed](HoprToken.md#_deployed)
- [\_wrapEvent](HoprToken.md#_wrapevent)
- [accountSnapshots](HoprToken.md#accountsnapshots)
- [allowance](HoprToken.md#allowance)
- [approve](HoprToken.md#approve)
- [attach](HoprToken.md#attach)
- [authorizeOperator](HoprToken.md#authorizeoperator)
- [balanceOf](HoprToken.md#balanceof)
- [balanceOfAt](HoprToken.md#balanceofat)
- [burn](HoprToken.md#burn)
- [connect](HoprToken.md#connect)
- [decimals](HoprToken.md#decimals)
- [defaultOperators](HoprToken.md#defaultoperators)
- [deployed](HoprToken.md#deployed)
- [emit](HoprToken.md#emit)
- [fallback](HoprToken.md#fallback)
- [getRoleAdmin](HoprToken.md#getroleadmin)
- [getRoleMember](HoprToken.md#getrolemember)
- [getRoleMemberCount](HoprToken.md#getrolemembercount)
- [grantRole](HoprToken.md#grantrole)
- [granularity](HoprToken.md#granularity)
- [hasRole](HoprToken.md#hasrole)
- [isOperatorFor](HoprToken.md#isoperatorfor)
- [listenerCount](HoprToken.md#listenercount)
- [listeners](HoprToken.md#listeners)
- [mint](HoprToken.md#mint)
- [name](HoprToken.md#name)
- [off](HoprToken.md#off)
- [on](HoprToken.md#on)
- [once](HoprToken.md#once)
- [operatorBurn](HoprToken.md#operatorburn)
- [operatorSend](HoprToken.md#operatorsend)
- [queryFilter](HoprToken.md#queryfilter)
- [removeAllListeners](HoprToken.md#removealllisteners)
- [removeListener](HoprToken.md#removelistener)
- [renounceRole](HoprToken.md#renouncerole)
- [revokeOperator](HoprToken.md#revokeoperator)
- [revokeRole](HoprToken.md#revokerole)
- [send](HoprToken.md#send)
- [symbol](HoprToken.md#symbol)
- [totalSupply](HoprToken.md#totalsupply)
- [totalSupplyAt](HoprToken.md#totalsupplyat)
- [totalSupplySnapshots](HoprToken.md#totalsupplysnapshots)
- [transfer](HoprToken.md#transfer)
- [transferFrom](HoprToken.md#transferfrom)
- [getContractAddress](HoprToken.md#getcontractaddress)
- [getInterface](HoprToken.md#getinterface)
- [isIndexed](HoprToken.md#isindexed)

## Constructors

### constructor

• **new HoprToken**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

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
| `MINTER_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `accountSnapshots` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\> |
| `allowance` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `approve` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `authorizeOperator` | (`operator`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `balanceOf` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `balanceOfAt` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `burn` | (`amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<`number`\> |
| `defaultOperators` | (`overrides?`: `CallOverrides`) => `Promise`<`string`[]\> |
| `getRoleAdmin` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `getRoleMember` | (`role`: `BytesLike`, `index`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `getRoleMemberCount` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `grantRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `granularity` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `hasRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `isOperatorFor` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `mint` | (`account`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `operatorBurn` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `operatorSend` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `renounceRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `revokeOperator` | (`operator`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `revokeRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `send` | (`recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `symbol` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupplyAt` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupplySnapshots` | (`arg0`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\> |
| `transfer` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transferFrom` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |

#### Overrides

BaseContract.callStatic

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:725

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
| `MINTER_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `accountSnapshots` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `allowance` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `approve` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `authorizeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `balanceOf` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `balanceOfAt` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `burn` | (`amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `defaultOperators` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getRoleAdmin` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getRoleMember` | (`role`: `BytesLike`, `index`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getRoleMemberCount` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `grantRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `granularity` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `hasRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isOperatorFor` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `mint` | (`account`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `operatorBurn` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `operatorSend` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `renounceRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `revokeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `revokeRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `send` | (`recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `symbol` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupplyAt` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupplySnapshots` | (`arg0`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `transfer` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferFrom` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

BaseContract.estimateGas

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:1117

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Approval` | (`owner?`: `string`, `spender?`: `string`, `value?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`], { `owner`: `string` ; `spender`: `string` ; `value`: `BigNumber`  }\> |
| `Approval(address,address,uint256)` | (`owner?`: `string`, `spender?`: `string`, `value?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`], { `owner`: `string` ; `spender`: `string` ; `value`: `BigNumber`  }\> |
| `AuthorizedOperator` | (`operator?`: `string`, `tokenHolder?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`], { `operator`: `string` ; `tokenHolder`: `string`  }\> |
| `AuthorizedOperator(address,address)` | (`operator?`: `string`, `tokenHolder?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`], { `operator`: `string` ; `tokenHolder`: `string`  }\> |
| `Burned` | (`operator?`: `string`, `from?`: `string`, `amount?`: ``null``, `data?`: ``null``, `operatorData?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`, `string`, `string`], { `amount`: `BigNumber` ; `data`: `string` ; `from`: `string` ; `operator`: `string` ; `operatorData`: `string`  }\> |
| `Burned(address,address,uint256,bytes,bytes)` | (`operator?`: `string`, `from?`: `string`, `amount?`: ``null``, `data?`: ``null``, `operatorData?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`, `string`, `string`], { `amount`: `BigNumber` ; `data`: `string` ; `from`: `string` ; `operator`: `string` ; `operatorData`: `string`  }\> |
| `Minted` | (`operator?`: `string`, `to?`: `string`, `amount?`: ``null``, `data?`: ``null``, `operatorData?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`, `string`, `string`], { `amount`: `BigNumber` ; `data`: `string` ; `operator`: `string` ; `operatorData`: `string` ; `to`: `string`  }\> |
| `Minted(address,address,uint256,bytes,bytes)` | (`operator?`: `string`, `to?`: `string`, `amount?`: ``null``, `data?`: ``null``, `operatorData?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`, `string`, `string`], { `amount`: `BigNumber` ; `data`: `string` ; `operator`: `string` ; `operatorData`: `string` ; `to`: `string`  }\> |
| `RevokedOperator` | (`operator?`: `string`, `tokenHolder?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`], { `operator`: `string` ; `tokenHolder`: `string`  }\> |
| `RevokedOperator(address,address)` | (`operator?`: `string`, `tokenHolder?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`], { `operator`: `string` ; `tokenHolder`: `string`  }\> |
| `RoleAdminChanged` | (`role?`: `BytesLike`, `previousAdminRole?`: `BytesLike`, `newAdminRole?`: `BytesLike`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`], { `newAdminRole`: `string` ; `previousAdminRole`: `string` ; `role`: `string`  }\> |
| `RoleAdminChanged(bytes32,bytes32,bytes32)` | (`role?`: `BytesLike`, `previousAdminRole?`: `BytesLike`, `newAdminRole?`: `BytesLike`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`], { `newAdminRole`: `string` ; `previousAdminRole`: `string` ; `role`: `string`  }\> |
| `RoleGranted` | (`role?`: `BytesLike`, `account?`: `string`, `sender?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`], { `account`: `string` ; `role`: `string` ; `sender`: `string`  }\> |
| `RoleGranted(bytes32,address,address)` | (`role?`: `BytesLike`, `account?`: `string`, `sender?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`], { `account`: `string` ; `role`: `string` ; `sender`: `string`  }\> |
| `RoleRevoked` | (`role?`: `BytesLike`, `account?`: `string`, `sender?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`], { `account`: `string` ; `role`: `string` ; `sender`: `string`  }\> |
| `RoleRevoked(bytes32,address,address)` | (`role?`: `BytesLike`, `account?`: `string`, `sender?`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`], { `account`: `string` ; `role`: `string` ; `sender`: `string`  }\> |
| `Sent` | (`operator?`: `string`, `from?`: `string`, `to?`: `string`, `amount?`: ``null``, `data?`: ``null``, `operatorData?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`, `BigNumber`, `string`, `string`], { `amount`: `BigNumber` ; `data`: `string` ; `from`: `string` ; `operator`: `string` ; `operatorData`: `string` ; `to`: `string`  }\> |
| `Sent(address,address,address,uint256,bytes,bytes)` | (`operator?`: `string`, `from?`: `string`, `to?`: `string`, `amount?`: ``null``, `data?`: ``null``, `operatorData?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`, `BigNumber`, `string`, `string`], { `amount`: `BigNumber` ; `data`: `string` ; `from`: `string` ; `operator`: `string` ; `operatorData`: `string` ; `to`: `string`  }\> |
| `Transfer` | (`from?`: `string`, `to?`: `string`, `value?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`], { `from`: `string` ; `to`: `string` ; `value`: `BigNumber`  }\> |
| `Transfer(address,address,uint256)` | (`from?`: `string`, `to?`: `string`, `value?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`], { `from`: `string` ; `to`: `string` ; `value`: `BigNumber`  }\> |

#### Overrides

BaseContract.filters

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:887

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `DEFAULT_ADMIN_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `MINTER_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `accountSnapshots` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\> |
| `allowance` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `approve` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `authorizeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `balanceOf` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `balanceOfAt` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `burn` | (`amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<[`number`]\> |
| `defaultOperators` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`[]]\> |
| `getRoleAdmin` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getRoleMember` | (`role`: `BytesLike`, `index`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getRoleMemberCount` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `grantRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `granularity` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `hasRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `isOperatorFor` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `mint` | (`account`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `operatorBurn` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `operatorSend` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `renounceRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `revokeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `revokeRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `send` | (`recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `symbol` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `totalSupplyAt` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `totalSupplySnapshots` | (`arg0`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\> |
| `transfer` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferFrom` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

BaseContract.functions

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:400

___

### interface

• **interface**: `HoprTokenInterface`

#### Overrides

BaseContract.interface

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:398

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `DEFAULT_ADMIN_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `MINTER_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `accountSnapshots` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `allowance` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `approve` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `authorizeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `balanceOf` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `balanceOfAt` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `burn` | (`amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `defaultOperators` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getRoleAdmin` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getRoleMember` | (`role`: `BytesLike`, `index`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getRoleMemberCount` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `grantRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `granularity` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `hasRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `isOperatorFor` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `mint` | (`account`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `operatorBurn` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `operatorSend` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `renounceRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `revokeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `revokeRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `send` | (`recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `symbol` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalSupplyAt` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalSupplySnapshots` | (`arg0`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `transfer` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferFrom` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

BaseContract.populateTransaction

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:1281

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

packages/ethereum/src/types/HoprToken.d.ts:565

___

### MINTER\_ROLE

▸ **MINTER_ROLE**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:567

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

### accountSnapshots

▸ **accountSnapshots**(`arg0`, `arg1`, `overrides?`): `Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `string` |
| `arg1` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:569

___

### allowance

▸ **allowance**(`holder`, `spender`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `holder` | `string` |
| `spender` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:577

___

### approve

▸ **approve**(`spender`, `value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `spender` | `string` |
| `value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:583

___

### attach

▸ **attach**(`addressOrName`): [`HoprToken`](HoprToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`HoprToken`](HoprToken.md)

#### Overrides

BaseContract.attach

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:359

___

### authorizeOperator

▸ **authorizeOperator**(`operator`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:589

___

### balanceOf

▸ **balanceOf**(`tokenHolder`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `tokenHolder` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:594

___

### balanceOfAt

▸ **balanceOfAt**(`_owner`, `_blockNumber`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | `string` |
| `_blockNumber` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:596

___

### burn

▸ **burn**(`amount`, `data`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `amount` | `BigNumberish` |
| `data` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:602

___

### connect

▸ **connect**(`signerOrProvider`): [`HoprToken`](HoprToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`HoprToken`](HoprToken.md)

#### Overrides

BaseContract.connect

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:358

___

### decimals

▸ **decimals**(`overrides?`): `Promise`<`number`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`number`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:608

___

### defaultOperators

▸ **defaultOperators**(`overrides?`): `Promise`<`string`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`[]\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:610

___

### deployed

▸ **deployed**(): `Promise`<[`HoprToken`](HoprToken.md)\>

#### Returns

`Promise`<[`HoprToken`](HoprToken.md)\>

#### Overrides

BaseContract.deployed

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:360

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

packages/ethereum/src/types/HoprToken.d.ts:612

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

packages/ethereum/src/types/HoprToken.d.ts:614

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

packages/ethereum/src/types/HoprToken.d.ts:620

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

packages/ethereum/src/types/HoprToken.d.ts:625

___

### granularity

▸ **granularity**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:631

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

packages/ethereum/src/types/HoprToken.d.ts:633

___

### isOperatorFor

▸ **isOperatorFor**(`operator`, `tokenHolder`, `overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | `string` |
| `tokenHolder` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:639

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

packages/ethereum/src/types/HoprToken.d.ts:362

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

packages/ethereum/src/types/HoprToken.d.ts:385

___

### mint

▸ **mint**(`account`, `amount`, `userData`, `operatorData`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `amount` | `BigNumberish` |
| `userData` | `BytesLike` |
| `operatorData` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:645

___

### name

▸ **name**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:653

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprToken`](HoprToken.md)

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

[`HoprToken`](HoprToken.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:365

▸ **off**(`eventName`, `listener`): [`HoprToken`](HoprToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprToken`](HoprToken.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:386

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprToken`](HoprToken.md)

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

[`HoprToken`](HoprToken.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:369

▸ **on**(`eventName`, `listener`): [`HoprToken`](HoprToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprToken`](HoprToken.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:387

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprToken`](HoprToken.md)

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

[`HoprToken`](HoprToken.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:373

▸ **once**(`eventName`, `listener`): [`HoprToken`](HoprToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprToken`](HoprToken.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:388

___

### operatorBurn

▸ **operatorBurn**(`account`, `amount`, `data`, `operatorData`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `amount` | `BigNumberish` |
| `data` | `BytesLike` |
| `operatorData` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:655

___

### operatorSend

▸ **operatorSend**(`sender`, `recipient`, `amount`, `data`, `operatorData`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `sender` | `string` |
| `recipient` | `string` |
| `amount` | `BigNumberish` |
| `data` | `BytesLike` |
| `operatorData` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:663

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

packages/ethereum/src/types/HoprToken.d.ts:392

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`HoprToken`](HoprToken.md)

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

[`HoprToken`](HoprToken.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:381

▸ **removeAllListeners**(`eventName?`): [`HoprToken`](HoprToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`HoprToken`](HoprToken.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:390

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprToken`](HoprToken.md)

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

[`HoprToken`](HoprToken.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:377

▸ **removeListener**(`eventName`, `listener`): [`HoprToken`](HoprToken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprToken`](HoprToken.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:389

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

packages/ethereum/src/types/HoprToken.d.ts:672

___

### revokeOperator

▸ **revokeOperator**(`operator`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:678

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

packages/ethereum/src/types/HoprToken.d.ts:683

___

### send

▸ **send**(`recipient`, `amount`, `data`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `recipient` | `string` |
| `amount` | `BigNumberish` |
| `data` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:689

___

### symbol

▸ **symbol**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:696

___

### totalSupply

▸ **totalSupply**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:698

___

### totalSupplyAt

▸ **totalSupplyAt**(`_blockNumber`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_blockNumber` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:700

___

### totalSupplySnapshots

▸ **totalSupplySnapshots**(`arg0`, `overrides?`): `Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:705

___

### transfer

▸ **transfer**(`recipient`, `amount`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `recipient` | `string` |
| `amount` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:712

___

### transferFrom

▸ **transferFrom**(`holder`, `recipient`, `amount`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `holder` | `string` |
| `recipient` | `string` |
| `amount` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/HoprToken.d.ts:718

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
