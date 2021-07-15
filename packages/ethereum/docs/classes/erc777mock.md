[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ERC777Mock

# Class: ERC777Mock

## Hierarchy

- `Contract`

  ↳ **`ERC777Mock`**

## Table of contents

### Constructors

- [constructor](erc777mock.md#constructor)

### Properties

- [\_deployedPromise](erc777mock.md#_deployedpromise)
- [\_runningEvents](erc777mock.md#_runningevents)
- [\_wrappedEmits](erc777mock.md#_wrappedemits)
- [address](erc777mock.md#address)
- [callStatic](erc777mock.md#callstatic)
- [deployTransaction](erc777mock.md#deploytransaction)
- [estimateGas](erc777mock.md#estimategas)
- [filters](erc777mock.md#filters)
- [functions](erc777mock.md#functions)
- [interface](erc777mock.md#interface)
- [populateTransaction](erc777mock.md#populatetransaction)
- [provider](erc777mock.md#provider)
- [resolvedAddress](erc777mock.md#resolvedaddress)
- [signer](erc777mock.md#signer)

### Methods

- [\_checkRunningEvents](erc777mock.md#_checkrunningevents)
- [\_deployed](erc777mock.md#_deployed)
- [\_wrapEvent](erc777mock.md#_wrapevent)
- [allowance](erc777mock.md#allowance)
- [allowance(address,address)](erc777mock.md#allowance(address,address))
- [approve](erc777mock.md#approve)
- [approve(address,uint256)](erc777mock.md#approve(address,uint256))
- [approveInternal](erc777mock.md#approveinternal)
- [approveInternal(address,address,uint256)](erc777mock.md#approveinternal(address,address,uint256))
- [attach](erc777mock.md#attach)
- [authorizeOperator](erc777mock.md#authorizeoperator)
- [authorizeOperator(address)](erc777mock.md#authorizeoperator(address))
- [balanceOf](erc777mock.md#balanceof)
- [balanceOf(address)](erc777mock.md#balanceof(address))
- [burn](erc777mock.md#burn)
- [burn(uint256,bytes)](erc777mock.md#burn(uint256,bytes))
- [connect](erc777mock.md#connect)
- [decimals](erc777mock.md#decimals)
- [decimals()](erc777mock.md#decimals())
- [defaultOperators](erc777mock.md#defaultoperators)
- [defaultOperators()](erc777mock.md#defaultoperators())
- [deployed](erc777mock.md#deployed)
- [emit](erc777mock.md#emit)
- [fallback](erc777mock.md#fallback)
- [granularity](erc777mock.md#granularity)
- [granularity()](erc777mock.md#granularity())
- [isOperatorFor](erc777mock.md#isoperatorfor)
- [isOperatorFor(address,address)](erc777mock.md#isoperatorfor(address,address))
- [listenerCount](erc777mock.md#listenercount)
- [listeners](erc777mock.md#listeners)
- [mintInternal](erc777mock.md#mintinternal)
- [mintInternal(address,uint256,bytes,bytes)](erc777mock.md#mintinternal(address,uint256,bytes,bytes))
- [name](erc777mock.md#name)
- [name()](erc777mock.md#name())
- [off](erc777mock.md#off)
- [on](erc777mock.md#on)
- [once](erc777mock.md#once)
- [operatorBurn](erc777mock.md#operatorburn)
- [operatorBurn(address,uint256,bytes,bytes)](erc777mock.md#operatorburn(address,uint256,bytes,bytes))
- [operatorSend](erc777mock.md#operatorsend)
- [operatorSend(address,address,uint256,bytes,bytes)](erc777mock.md#operatorsend(address,address,uint256,bytes,bytes))
- [queryFilter](erc777mock.md#queryfilter)
- [removeAllListeners](erc777mock.md#removealllisteners)
- [removeListener](erc777mock.md#removelistener)
- [revokeOperator](erc777mock.md#revokeoperator)
- [revokeOperator(address)](erc777mock.md#revokeoperator(address))
- [send](erc777mock.md#send)
- [send(address,uint256,bytes)](erc777mock.md#send(address,uint256,bytes))
- [symbol](erc777mock.md#symbol)
- [symbol()](erc777mock.md#symbol())
- [totalSupply](erc777mock.md#totalsupply)
- [totalSupply()](erc777mock.md#totalsupply())
- [transfer](erc777mock.md#transfer)
- [transfer(address,uint256)](erc777mock.md#transfer(address,uint256))
- [transferFrom](erc777mock.md#transferfrom)
- [transferFrom(address,address,uint256)](erc777mock.md#transferfrom(address,address,uint256))
- [getContractAddress](erc777mock.md#getcontractaddress)
- [getInterface](erc777mock.md#getinterface)
- [isIndexed](erc777mock.md#isindexed)

## Constructors

### constructor

• **new ERC777Mock**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

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
| `allowance` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `allowance(address,address)` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `approve` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `approve(address,uint256)` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `approveInternal` | (`holder`: `string`, `spender`: `string`, `value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `approveInternal(address,address,uint256)` | (`holder`: `string`, `spender`: `string`, `value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `authorizeOperator` | (`operator`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `authorizeOperator(address)` | (`operator`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `balanceOf` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `balanceOf(address)` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
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
| `mintInternal` | (`to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `mintInternal(address,uint256,bytes,bytes)` | (`to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
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
| `transfer` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transfer(address,uint256)` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transferFrom` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `transferFrom(address,address,uint256)` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |

#### Overrides

Contract.callStatic

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:642

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
| `allowance` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `allowance(address,address)` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `approve` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `approve(address,uint256)` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `approveInternal` | (`holder`: `string`, `spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `approveInternal(address,address,uint256)` | (`holder`: `string`, `spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `authorizeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `authorizeOperator(address)` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `balanceOf` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `balanceOf(address)` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
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
| `mintInternal` | (`to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `mintInternal(address,uint256,bytes,bytes)` | (`to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
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
| `transfer` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transfer(address,uint256)` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferFrom` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `transferFrom(address,address,uint256)` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

Contract.estimateGas

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:936

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

packages/ethereum/types/ERC777Mock.d.ts:847

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `allowance` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `allowance(address,address)` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `approve` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `approve(address,uint256)` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `approveInternal` | (`holder`: `string`, `spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `approveInternal(address,address,uint256)` | (`holder`: `string`, `spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `authorizeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `authorizeOperator(address)` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `balanceOf` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `balanceOf(address)` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
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
| `mintInternal` | (`to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `mintInternal(address,uint256,bytes,bytes)` | (`to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
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
| `transfer` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transfer(address,uint256)` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferFrom` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `transferFrom(address,address,uint256)` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

Contract.functions

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:231

___

### interface

• **interface**: `ERC777MockInterface`

#### Overrides

Contract.interface

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:229

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `allowance` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `allowance(address,address)` | (`holder`: `string`, `spender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `approve` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `approve(address,uint256)` | (`spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `approveInternal` | (`holder`: `string`, `spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `approveInternal(address,address,uint256)` | (`holder`: `string`, `spender`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `authorizeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `authorizeOperator(address)` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `balanceOf` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `balanceOf(address)` | (`tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
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
| `mintInternal` | (`to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `mintInternal(address,uint256,bytes,bytes)` | (`to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
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
| `transfer` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transfer(address,uint256)` | (`recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferFrom` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `transferFrom(address,address,uint256)` | (`holder`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

Contract.populateTransaction

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:1144

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

packages/ethereum/types/ERC777Mock.d.ts:439

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

packages/ethereum/types/ERC777Mock.d.ts:443

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

packages/ethereum/types/ERC777Mock.d.ts:451

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

packages/ethereum/types/ERC777Mock.d.ts:455

___

### approveInternal

▸ **approveInternal**(`holder`, `spender`, `value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `holder` | `string` |
| `spender` | `string` |
| `value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:463

___

### approveInternal(address,address,uint256)

▸ **approveInternal(address,address,uint256)**(`holder`, `spender`, `value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `holder` | `string` |
| `spender` | `string` |
| `value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:468

___

### attach

▸ **attach**(`addressOrName`): [`ERC777Mock`](erc777mock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`ERC777Mock`](erc777mock.md)

#### Overrides

Contract.attach

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:190

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

packages/ethereum/types/ERC777Mock.d.ts:477

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

packages/ethereum/types/ERC777Mock.d.ts:480

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

packages/ethereum/types/ERC777Mock.d.ts:487

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

packages/ethereum/types/ERC777Mock.d.ts:487

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

packages/ethereum/types/ERC777Mock.d.ts:494

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

packages/ethereum/types/ERC777Mock.d.ts:498

___

### connect

▸ **connect**(`signerOrProvider`): [`ERC777Mock`](erc777mock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`ERC777Mock`](erc777mock.md)

#### Overrides

Contract.connect

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:189

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

packages/ethereum/types/ERC777Mock.d.ts:506

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

packages/ethereum/types/ERC777Mock.d.ts:506

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

packages/ethereum/types/ERC777Mock.d.ts:510

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

packages/ethereum/types/ERC777Mock.d.ts:510

___

### deployed

▸ **deployed**(): `Promise`<[`ERC777Mock`](erc777mock.md)\>

#### Returns

`Promise`<[`ERC777Mock`](erc777mock.md)\>

#### Overrides

Contract.deployed

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:191

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

packages/ethereum/types/ERC777Mock.d.ts:514

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

packages/ethereum/types/ERC777Mock.d.ts:514

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

packages/ethereum/types/ERC777Mock.d.ts:518

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

packages/ethereum/types/ERC777Mock.d.ts:522

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

packages/ethereum/types/ERC777Mock.d.ts:193

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

packages/ethereum/types/ERC777Mock.d.ts:216

___

### mintInternal

▸ **mintInternal**(`to`, `amount`, `userData`, `operatorData`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `to` | `string` |
| `amount` | `BigNumberish` |
| `userData` | `BytesLike` |
| `operatorData` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:530

___

### mintInternal(address,uint256,bytes,bytes)

▸ **mintInternal(address,uint256,bytes,bytes)**(`to`, `amount`, `userData`, `operatorData`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `to` | `string` |
| `amount` | `BigNumberish` |
| `userData` | `BytesLike` |
| `operatorData` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:536

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

packages/ethereum/types/ERC777Mock.d.ts:546

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

packages/ethereum/types/ERC777Mock.d.ts:546

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC777Mock`](erc777mock.md)

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

[`ERC777Mock`](erc777mock.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:196

▸ **off**(`eventName`, `listener`): [`ERC777Mock`](erc777mock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC777Mock`](erc777mock.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:217

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC777Mock`](erc777mock.md)

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

[`ERC777Mock`](erc777mock.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:200

▸ **on**(`eventName`, `listener`): [`ERC777Mock`](erc777mock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC777Mock`](erc777mock.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:218

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC777Mock`](erc777mock.md)

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

[`ERC777Mock`](erc777mock.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:204

▸ **once**(`eventName`, `listener`): [`ERC777Mock`](erc777mock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC777Mock`](erc777mock.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:219

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

packages/ethereum/types/ERC777Mock.d.ts:550

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

packages/ethereum/types/ERC777Mock.d.ts:556

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

packages/ethereum/types/ERC777Mock.d.ts:566

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

packages/ethereum/types/ERC777Mock.d.ts:573

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

packages/ethereum/types/ERC777Mock.d.ts:223

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`ERC777Mock`](erc777mock.md)

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

[`ERC777Mock`](erc777mock.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:212

▸ **removeAllListeners**(`eventName?`): [`ERC777Mock`](erc777mock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`ERC777Mock`](erc777mock.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:221

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC777Mock`](erc777mock.md)

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

[`ERC777Mock`](erc777mock.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:208

▸ **removeListener**(`eventName`, `listener`): [`ERC777Mock`](erc777mock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC777Mock`](erc777mock.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/ERC777Mock.d.ts:220

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

packages/ethereum/types/ERC777Mock.d.ts:584

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

packages/ethereum/types/ERC777Mock.d.ts:587

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

packages/ethereum/types/ERC777Mock.d.ts:594

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

packages/ethereum/types/ERC777Mock.d.ts:599

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

packages/ethereum/types/ERC777Mock.d.ts:608

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

packages/ethereum/types/ERC777Mock.d.ts:608

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

packages/ethereum/types/ERC777Mock.d.ts:612

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

packages/ethereum/types/ERC777Mock.d.ts:612

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

packages/ethereum/types/ERC777Mock.d.ts:616

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

packages/ethereum/types/ERC777Mock.d.ts:620

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

packages/ethereum/types/ERC777Mock.d.ts:628

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

packages/ethereum/types/ERC777Mock.d.ts:633

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
