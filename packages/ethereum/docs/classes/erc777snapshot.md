[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ERC777Snapshot

# Class: ERC777Snapshot

## Hierarchy

- `Contract`

  ↳ **`ERC777Snapshot`**

## Table of contents

### Constructors

- [constructor](erc777snapshot.md#constructor)

### Properties

- [\_deployedPromise](erc777snapshot.md#_deployedpromise)
- [\_runningEvents](erc777snapshot.md#_runningevents)
- [\_wrappedEmits](erc777snapshot.md#_wrappedemits)
- [address](erc777snapshot.md#address)
- [callStatic](erc777snapshot.md#callstatic)
- [deployTransaction](erc777snapshot.md#deploytransaction)
- [estimateGas](erc777snapshot.md#estimategas)
- [filters](erc777snapshot.md#filters)
- [functions](erc777snapshot.md#functions)
- [interface](erc777snapshot.md#interface)
- [populateTransaction](erc777snapshot.md#populatetransaction)
- [provider](erc777snapshot.md#provider)
- [resolvedAddress](erc777snapshot.md#resolvedaddress)
- [signer](erc777snapshot.md#signer)

### Methods

- [\_checkRunningEvents](erc777snapshot.md#_checkrunningevents)
- [\_deployed](erc777snapshot.md#_deployed)
- [\_wrapEvent](erc777snapshot.md#_wrapevent)
- [accountSnapshots](erc777snapshot.md#accountsnapshots)
- [accountSnapshots(address,uint256)](erc777snapshot.md#accountsnapshots(address,uint256))
- [allowance](erc777snapshot.md#allowance)
- [allowance(address,address)](erc777snapshot.md#allowance(address,address))
- [approve](erc777snapshot.md#approve)
- [approve(address,uint256)](erc777snapshot.md#approve(address,uint256))
- [attach](erc777snapshot.md#attach)
- [authorizeOperator](erc777snapshot.md#authorizeoperator)
- [authorizeOperator(address)](erc777snapshot.md#authorizeoperator(address))
- [balanceOf](erc777snapshot.md#balanceof)
- [balanceOf(address)](erc777snapshot.md#balanceof(address))
- [balanceOfAt](erc777snapshot.md#balanceofat)
- [balanceOfAt(address,uint128)](erc777snapshot.md#balanceofat(address,uint128))
- [burn](erc777snapshot.md#burn)
- [burn(uint256,bytes)](erc777snapshot.md#burn(uint256,bytes))
- [connect](erc777snapshot.md#connect)
- [decimals](erc777snapshot.md#decimals)
- [decimals()](erc777snapshot.md#decimals())
- [defaultOperators](erc777snapshot.md#defaultoperators)
- [defaultOperators()](erc777snapshot.md#defaultoperators())
- [deployed](erc777snapshot.md#deployed)
- [emit](erc777snapshot.md#emit)
- [fallback](erc777snapshot.md#fallback)
- [granularity](erc777snapshot.md#granularity)
- [granularity()](erc777snapshot.md#granularity())
- [isOperatorFor](erc777snapshot.md#isoperatorfor)
- [isOperatorFor(address,address)](erc777snapshot.md#isoperatorfor(address,address))
- [listenerCount](erc777snapshot.md#listenercount)
- [listeners](erc777snapshot.md#listeners)
- [name](erc777snapshot.md#name)
- [name()](erc777snapshot.md#name())
- [off](erc777snapshot.md#off)
- [on](erc777snapshot.md#on)
- [once](erc777snapshot.md#once)
- [operatorBurn](erc777snapshot.md#operatorburn)
- [operatorBurn(address,uint256,bytes,bytes)](erc777snapshot.md#operatorburn(address,uint256,bytes,bytes))
- [operatorSend](erc777snapshot.md#operatorsend)
- [operatorSend(address,address,uint256,bytes,bytes)](erc777snapshot.md#operatorsend(address,address,uint256,bytes,bytes))
- [queryFilter](erc777snapshot.md#queryfilter)
- [removeAllListeners](erc777snapshot.md#removealllisteners)
- [removeListener](erc777snapshot.md#removelistener)
- [revokeOperator](erc777snapshot.md#revokeoperator)
- [revokeOperator(address)](erc777snapshot.md#revokeoperator(address))
- [send](erc777snapshot.md#send)
- [send(address,uint256,bytes)](erc777snapshot.md#send(address,uint256,bytes))
- [symbol](erc777snapshot.md#symbol)
- [symbol()](erc777snapshot.md#symbol())
- [totalSupply](erc777snapshot.md#totalsupply)
- [totalSupply()](erc777snapshot.md#totalsupply())
- [totalSupplyAt](erc777snapshot.md#totalsupplyat)
- [totalSupplyAt(uint128)](erc777snapshot.md#totalsupplyat(uint128))
- [totalSupplySnapshots](erc777snapshot.md#totalsupplysnapshots)
- [totalSupplySnapshots(uint256)](erc777snapshot.md#totalsupplysnapshots(uint256))
- [transfer](erc777snapshot.md#transfer)
- [transfer(address,uint256)](erc777snapshot.md#transfer(address,uint256))
- [transferFrom](erc777snapshot.md#transferfrom)
- [transferFrom(address,address,uint256)](erc777snapshot.md#transferfrom(address,address,uint256))
- [getContractAddress](erc777snapshot.md#getcontractaddress)
- [getInterface](erc777snapshot.md#getinterface)
- [isIndexed](erc777snapshot.md#isindexed)

## Constructors

### constructor

• **new ERC777Snapshot**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |
| `contractInterface` | `ContractInterface` |
| `signerOrProvider?` | `Signer` \| `Provider` |

#### Inherited from

Contract.constructor

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:102

## Properties

### \_deployedPromise

• **\_deployedPromise**: `Promise`<`Contract`\>

#### Inherited from

Contract.\_deployedPromise

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:96

___

### \_runningEvents

• **\_runningEvents**: `Object`

#### Index signature

▪ [eventTag: `string`]: `RunningEvent`

#### Inherited from

Contract.\_runningEvents

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:97

___

### \_wrappedEmits

• **\_wrappedEmits**: `Object`

#### Index signature

▪ [eventTag: `string`]: (...`args`: `any`[]) => `void`

#### Inherited from

Contract.\_wrappedEmits

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:100

___

### address

• `Readonly` **address**: `string`

#### Inherited from

Contract.address

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:75

___

### callStatic

• **callStatic**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `accountSnapshots` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\> |
| `accountSnapshots(address,uint256)` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\> |
| `allowance` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `allowance(address,address)` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `approve` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `approve(address,uint256)` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `authorizeOperator` | (`operator`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `authorizeOperator(address)` | (`operator`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `balanceOf` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `balanceOf(address)` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `balanceOfAt` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `balanceOfAt(address,uint128)` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `burn` | (`amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `burn(uint256,bytes)` | (`amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<`number`\> |
| `decimals()` | (`overrides?`: `CallOverrides`) => `Promise`<`number`\> |
| `defaultOperators` | (`overrides?`: `CallOverrides`) => `Promise`<`string`[]\> |
| `defaultOperators()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`[]\> |
| `granularity` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `granularity()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isOperatorFor` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `isOperatorFor(address,address)` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `name()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `operatorBurn` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `operatorBurn(address,uint256,bytes,bytes)` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `operatorSend` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `operatorSend(address,address,uint256,bytes,bytes)` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `revokeOperator` | (`operator`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `revokeOperator(address)` | (`operator`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `send` | (`recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `send(address,uint256,bytes)` | (`recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `symbol` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `symbol()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupply()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupplyAt` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupplyAt(uint128)` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupplySnapshots` | (`arg0`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\> |
| `totalSupplySnapshots(uint256)` | (`arg0`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\> |
| `transfer` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transfer(address,uint256)` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transferFrom` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transferFrom(address,address,uint256)` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |

#### Overrides

Contract.callStatic

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:704

___

### deployTransaction

• `Readonly` **deployTransaction**: `TransactionResponse`

#### Inherited from

Contract.deployTransaction

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:95

___

### estimateGas

• **estimateGas**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `accountSnapshots` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `accountSnapshots(address,uint256)` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `allowance` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `allowance(address,address)` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `approve` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `approve(address,uint256)` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `authorizeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `authorizeOperator(address)` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `balanceOf` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `balanceOf(address)` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `balanceOfAt` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `balanceOfAt(address,uint128)` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `burn` | (`amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `burn(uint256,bytes)` | (`amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `decimals()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `defaultOperators` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `defaultOperators()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `granularity` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `granularity()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isOperatorFor` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isOperatorFor(address,address)` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `name()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `operatorBurn` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `operatorBurn(address,uint256,bytes,bytes)` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `operatorSend` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `operatorSend(address,address,uint256,bytes,bytes)` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `revokeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `revokeOperator(address)` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `send` | (`recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `send(address,uint256,bytes)` | (`recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `symbol` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `symbol()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupply()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupplyAt` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupplyAt(uint128)` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupplySnapshots` | (`arg0`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `totalSupplySnapshots(uint256)` | (`arg0`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `transfer` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transfer(address,uint256)` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferFrom` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferFrom(address,address,uint256)` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

Contract.estimateGas

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:1020

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Approval` | (`owner`: `string`, `spender`: `string`, `value`: ``null``) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`, `BigNumber`], `Object`\> |
| `AuthorizedOperator` | (`operator`: `string`, `tokenHolder`: `string`) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`], `Object`\> |
| `Burned` | (`operator`: `string`, `from`: `string`, `amount`: ``null``, `data`: ``null``, `operatorData`: ``null``) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`, `BigNumber`, `string`, `string`], `Object`\> |
| `Minted` | (`operator`: `string`, `to`: `string`, `amount`: ``null``, `data`: ``null``, `operatorData`: ``null``) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`, `BigNumber`, `string`, `string`], `Object`\> |
| `RevokedOperator` | (`operator`: `string`, `tokenHolder`: `string`) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`], `Object`\> |
| `Sent` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: ``null``, `data`: ``null``, `operatorData`: ``null``) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`, `string`, `BigNumber`, `string`, `string`], `Object`\> |
| `Transfer` | (`from`: `string`, `to`: `string`, `value`: ``null``) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`, `BigNumber`], `Object`\> |

#### Overrides

Contract.filters

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:931

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `accountSnapshots` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\> |
| `accountSnapshots(address,uint256)` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\> |
| `allowance` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `allowance(address,address)` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `approve` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `approve(address,uint256)` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `authorizeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `authorizeOperator(address)` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `balanceOf` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `balanceOf(address)` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `balanceOfAt` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `balanceOfAt(address,uint128)` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `burn` | (`amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `burn(uint256,bytes)` | (`amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<[`number`]\> |
| `decimals()` | (`overrides?`: `CallOverrides`) => `Promise`<[`number`]\> |
| `defaultOperators` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`[]]\> |
| `defaultOperators()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`[]]\> |
| `granularity` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `granularity()` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `isOperatorFor` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `isOperatorFor(address,address)` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `name()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `operatorBurn` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `operatorBurn(address,uint256,bytes,bytes)` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `operatorSend` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `operatorSend(address,address,uint256,bytes,bytes)` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `revokeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `revokeOperator(address)` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `send` | (`recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `send(address,uint256,bytes)` | (`recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `symbol` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `symbol()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `totalSupply()` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `totalSupplyAt` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `totalSupplyAt(uint128)` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `totalSupplySnapshots` | (`arg0`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\> |
| `totalSupplySnapshots(uint256)` | (`arg0`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\> |
| `transfer` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transfer(address,uint256)` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferFrom` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferFrom(address,address,uint256)` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

Contract.functions

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:249

___

### interface

• **interface**: `ERC777SnapshotInterface`

#### Overrides

Contract.interface

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:247

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `accountSnapshots` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `accountSnapshots(address,uint256)` | (`arg0`: `string`, `arg1`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `allowance` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `allowance(address,address)` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `approve` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `approve(address,uint256)` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `authorizeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `authorizeOperator(address)` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `balanceOf` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `balanceOf(address)` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `balanceOfAt` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `balanceOfAt(address,uint128)` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `burn` | (`amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `burn(uint256,bytes)` | (`amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `decimals()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `defaultOperators` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `defaultOperators()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `granularity` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `granularity()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `isOperatorFor` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `isOperatorFor(address,address)` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `name()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `operatorBurn` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `operatorBurn(address,uint256,bytes,bytes)` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `operatorSend` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `operatorSend(address,address,uint256,bytes,bytes)` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `revokeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `revokeOperator(address)` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `send` | (`recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `send(address,uint256,bytes)` | (`recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `symbol` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `symbol()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalSupply` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalSupply()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalSupplyAt` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalSupplyAt(uint128)` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalSupplySnapshots` | (`arg0`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `totalSupplySnapshots(uint256)` | (`arg0`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `transfer` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transfer(address,uint256)` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferFrom` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferFrom(address,address,uint256)` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

Contract.populateTransaction

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:1242

___

### provider

• `Readonly` **provider**: `Provider`

#### Inherited from

Contract.provider

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:78

___

### resolvedAddress

• `Readonly` **resolvedAddress**: `Promise`<`string`\>

#### Inherited from

Contract.resolvedAddress

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:94

___

### signer

• `Readonly` **signer**: `Signer`

#### Inherited from

Contract.signer

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:77

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

Contract.\_checkRunningEvents

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:117

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

Contract.\_deployed

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:110

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

Contract.\_wrapEvent

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:118

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

packages/ethereum/types/ERC777Snapshot.d.ts:479

___

### accountSnapshots(address,uint256)

▸ **accountSnapshots(address,uint256)**(`arg0`, `arg1`, `overrides?`): `Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `string` |
| `arg1` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:485

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

packages/ethereum/types/ERC777Snapshot.d.ts:495

___

### allowance(address,address)

▸ **allowance(address,address)**(`holder`, `spender`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `holder` | `string` |
| `spender` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:499

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

packages/ethereum/types/ERC777Snapshot.d.ts:507

___

### approve(address,uint256)

▸ **approve(address,uint256)**(`spender`, `value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `spender` | `string` |
| `value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:511

___

### attach

▸ **attach**(`addressOrName`): [`ERC777Snapshot`](erc777snapshot.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`ERC777Snapshot`](erc777snapshot.md)

#### Overrides

Contract.attach

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:208

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

packages/ethereum/types/ERC777Snapshot.d.ts:519

___

### authorizeOperator(address)

▸ **authorizeOperator(address)**(`operator`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:522

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

packages/ethereum/types/ERC777Snapshot.d.ts:529

___

### balanceOf(address)

▸ **balanceOf(address)**(`tokenHolder`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `tokenHolder` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:529

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

packages/ethereum/types/ERC777Snapshot.d.ts:536

___

### balanceOfAt(address,uint128)

▸ **balanceOfAt(address,uint128)**(`_owner`, `_blockNumber`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | `string` |
| `_blockNumber` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:540

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

packages/ethereum/types/ERC777Snapshot.d.ts:548

___

### burn(uint256,bytes)

▸ **burn(uint256,bytes)**(`amount`, `data`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `amount` | `BigNumberish` |
| `data` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:552

___

### connect

▸ **connect**(`signerOrProvider`): [`ERC777Snapshot`](erc777snapshot.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`ERC777Snapshot`](erc777snapshot.md)

#### Overrides

Contract.connect

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:207

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

packages/ethereum/types/ERC777Snapshot.d.ts:560

___

### decimals()

▸ **decimals()**(`overrides?`): `Promise`<`number`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`number`\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:560

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

packages/ethereum/types/ERC777Snapshot.d.ts:564

___

### defaultOperators()

▸ **defaultOperators()**(`overrides?`): `Promise`<`string`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`[]\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:564

___

### deployed

▸ **deployed**(): `Promise`<[`ERC777Snapshot`](erc777snapshot.md)\>

#### Returns

`Promise`<[`ERC777Snapshot`](erc777snapshot.md)\>

#### Overrides

Contract.deployed

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:209

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

Contract.emit

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:123

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

Contract.fallback

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:111

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

packages/ethereum/types/ERC777Snapshot.d.ts:568

___

### granularity()

▸ **granularity()**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:568

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

packages/ethereum/types/ERC777Snapshot.d.ts:572

___

### isOperatorFor(address,address)

▸ **isOperatorFor(address,address)**(`operator`, `tokenHolder`, `overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | `string` |
| `tokenHolder` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:576

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

Contract.listenerCount

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:124

___

### listeners

▸ **listeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter?`): [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\>[]

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter?` | [`TypedEventFilter`](../interfaces/typedeventfilter.md)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\>[]

#### Overrides

Contract.listeners

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:211

▸ **listeners**(`eventName?`): `Listener`[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

`Listener`[]

#### Overrides

Contract.listeners

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:234

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

packages/ethereum/types/ERC777Snapshot.d.ts:584

___

### name()

▸ **name()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:584

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC777Snapshot`](erc777snapshot.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/typedeventfilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`ERC777Snapshot`](erc777snapshot.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:214

▸ **off**(`eventName`, `listener`): [`ERC777Snapshot`](erc777snapshot.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC777Snapshot`](erc777snapshot.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:235

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC777Snapshot`](erc777snapshot.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/typedeventfilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`ERC777Snapshot`](erc777snapshot.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:218

▸ **on**(`eventName`, `listener`): [`ERC777Snapshot`](erc777snapshot.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC777Snapshot`](erc777snapshot.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:236

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC777Snapshot`](erc777snapshot.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/typedeventfilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`ERC777Snapshot`](erc777snapshot.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:222

▸ **once**(`eventName`, `listener`): [`ERC777Snapshot`](erc777snapshot.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC777Snapshot`](erc777snapshot.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:237

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

packages/ethereum/types/ERC777Snapshot.d.ts:588

___

### operatorBurn(address,uint256,bytes,bytes)

▸ **operatorBurn(address,uint256,bytes,bytes)**(`account`, `amount`, `data`, `operatorData`, `overrides?`): `Promise`<`ContractTransaction`\>

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

packages/ethereum/types/ERC777Snapshot.d.ts:594

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

packages/ethereum/types/ERC777Snapshot.d.ts:604

___

### operatorSend(address,address,uint256,bytes,bytes)

▸ **operatorSend(address,address,uint256,bytes,bytes)**(`sender`, `recipient`, `amount`, `data`, `operatorData`, `overrides?`): `Promise`<`ContractTransaction`\>

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

packages/ethereum/types/ERC777Snapshot.d.ts:611

___

### queryFilter

▸ **queryFilter**<`EventArgsArray`, `EventArgsObject`\>(`event`, `fromBlockOrBlockhash?`, `toBlock?`): `Promise`<[`TypedEvent`](../interfaces/typedevent.md)<`EventArgsArray` & `EventArgsObject`\>[]\>

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | [`TypedEventFilter`](../interfaces/typedeventfilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `fromBlockOrBlockhash?` | `string` \| `number` |
| `toBlock?` | `string` \| `number` |

#### Returns

`Promise`<[`TypedEvent`](../interfaces/typedevent.md)<`EventArgsArray` & `EventArgsObject`\>[]\>

#### Overrides

Contract.queryFilter

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:241

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`ERC777Snapshot`](erc777snapshot.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/typedeventfilter.md)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`ERC777Snapshot`](erc777snapshot.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:230

▸ **removeAllListeners**(`eventName?`): [`ERC777Snapshot`](erc777snapshot.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`ERC777Snapshot`](erc777snapshot.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:239

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC777Snapshot`](erc777snapshot.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/typedeventfilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`ERC777Snapshot`](erc777snapshot.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:226

▸ **removeListener**(`eventName`, `listener`): [`ERC777Snapshot`](erc777snapshot.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC777Snapshot`](erc777snapshot.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:238

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

packages/ethereum/types/ERC777Snapshot.d.ts:622

___

### revokeOperator(address)

▸ **revokeOperator(address)**(`operator`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:625

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

packages/ethereum/types/ERC777Snapshot.d.ts:632

___

### send(address,uint256,bytes)

▸ **send(address,uint256,bytes)**(`recipient`, `amount`, `data`, `overrides?`): `Promise`<`ContractTransaction`\>

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

packages/ethereum/types/ERC777Snapshot.d.ts:637

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

packages/ethereum/types/ERC777Snapshot.d.ts:646

___

### symbol()

▸ **symbol()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:646

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

packages/ethereum/types/ERC777Snapshot.d.ts:650

___

### totalSupply()

▸ **totalSupply()**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:650

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

packages/ethereum/types/ERC777Snapshot.d.ts:654

___

### totalSupplyAt(uint128)

▸ **totalSupplyAt(uint128)**(`_blockNumber`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_blockNumber` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:657

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

packages/ethereum/types/ERC777Snapshot.d.ts:664

___

### totalSupplySnapshots(uint256)

▸ **totalSupplySnapshots(uint256)**(`arg0`, `overrides?`): `Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`BigNumber`, `BigNumber`] & { `fromBlock`: `BigNumber` ; `value`: `BigNumber`  }\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:669

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

packages/ethereum/types/ERC777Snapshot.d.ts:678

___

### transfer(address,uint256)

▸ **transfer(address,uint256)**(`recipient`, `amount`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `recipient` | `string` |
| `amount` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC777Snapshot.d.ts:682

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

packages/ethereum/types/ERC777Snapshot.d.ts:690

___

### transferFrom(address,address,uint256)

▸ **transferFrom(address,address,uint256)**(`holder`, `recipient`, `amount`, `overrides?`): `Promise`<`ContractTransaction`\>

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

packages/ethereum/types/ERC777Snapshot.d.ts:695

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

Contract.getContractAddress

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:104

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

Contract.getInterface

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:108

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

Contract.isIndexed

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:114
