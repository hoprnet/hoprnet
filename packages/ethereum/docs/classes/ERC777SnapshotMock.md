[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ERC777SnapshotMock

# Class: ERC777SnapshotMock

## Hierarchy

- `Contract`

  ↳ **`ERC777SnapshotMock`**

## Table of contents

### Constructors

- [constructor](ERC777SnapshotMock.md#constructor)

### Properties

- [\_deployedPromise](ERC777SnapshotMock.md#_deployedpromise)
- [\_runningEvents](ERC777SnapshotMock.md#_runningevents)
- [\_wrappedEmits](ERC777SnapshotMock.md#_wrappedemits)
- [address](ERC777SnapshotMock.md#address)
- [callStatic](ERC777SnapshotMock.md#callstatic)
- [deployTransaction](ERC777SnapshotMock.md#deploytransaction)
- [estimateGas](ERC777SnapshotMock.md#estimategas)
- [filters](ERC777SnapshotMock.md#filters)
- [functions](ERC777SnapshotMock.md#functions)
- [interface](ERC777SnapshotMock.md#interface)
- [populateTransaction](ERC777SnapshotMock.md#populatetransaction)
- [provider](ERC777SnapshotMock.md#provider)
- [resolvedAddress](ERC777SnapshotMock.md#resolvedaddress)
- [signer](ERC777SnapshotMock.md#signer)

### Methods

- [\_checkRunningEvents](ERC777SnapshotMock.md#_checkrunningevents)
- [\_deployed](ERC777SnapshotMock.md#_deployed)
- [\_wrapEvent](ERC777SnapshotMock.md#_wrapevent)
- [accountSnapshots](ERC777SnapshotMock.md#accountsnapshots)
- [accountSnapshots(address,uint256)](ERC777SnapshotMock.md#accountsnapshots(address,uint256))
- [allowance](ERC777SnapshotMock.md#allowance)
- [allowance(address,address)](ERC777SnapshotMock.md#allowance(address,address))
- [approve](ERC777SnapshotMock.md#approve)
- [approve(address,uint256)](ERC777SnapshotMock.md#approve(address,uint256))
- [attach](ERC777SnapshotMock.md#attach)
- [authorizeOperator](ERC777SnapshotMock.md#authorizeoperator)
- [authorizeOperator(address)](ERC777SnapshotMock.md#authorizeoperator(address))
- [balanceOf](ERC777SnapshotMock.md#balanceof)
- [balanceOf(address)](ERC777SnapshotMock.md#balanceof(address))
- [balanceOfAt](ERC777SnapshotMock.md#balanceofat)
- [balanceOfAt(address,uint128)](ERC777SnapshotMock.md#balanceofat(address,uint128))
- [burn(address,uint256,bytes,bytes)](ERC777SnapshotMock.md#burn(address,uint256,bytes,bytes))
- [burn(uint256,bytes)](ERC777SnapshotMock.md#burn(uint256,bytes))
- [connect](ERC777SnapshotMock.md#connect)
- [decimals](ERC777SnapshotMock.md#decimals)
- [decimals()](ERC777SnapshotMock.md#decimals())
- [defaultOperators](ERC777SnapshotMock.md#defaultoperators)
- [defaultOperators()](ERC777SnapshotMock.md#defaultoperators())
- [deployed](ERC777SnapshotMock.md#deployed)
- [emit](ERC777SnapshotMock.md#emit)
- [fallback](ERC777SnapshotMock.md#fallback)
- [getAccountValueAt](ERC777SnapshotMock.md#getaccountvalueat)
- [getAccountValueAt(address,uint128)](ERC777SnapshotMock.md#getaccountvalueat(address,uint128))
- [getTotalSupplyValueAt](ERC777SnapshotMock.md#gettotalsupplyvalueat)
- [getTotalSupplyValueAt(uint128)](ERC777SnapshotMock.md#gettotalsupplyvalueat(uint128))
- [granularity](ERC777SnapshotMock.md#granularity)
- [granularity()](ERC777SnapshotMock.md#granularity())
- [isOperatorFor](ERC777SnapshotMock.md#isoperatorfor)
- [isOperatorFor(address,address)](ERC777SnapshotMock.md#isoperatorfor(address,address))
- [listenerCount](ERC777SnapshotMock.md#listenercount)
- [listeners](ERC777SnapshotMock.md#listeners)
- [mint](ERC777SnapshotMock.md#mint)
- [mint(address,uint256,bytes,bytes)](ERC777SnapshotMock.md#mint(address,uint256,bytes,bytes))
- [name](ERC777SnapshotMock.md#name)
- [name()](ERC777SnapshotMock.md#name())
- [off](ERC777SnapshotMock.md#off)
- [on](ERC777SnapshotMock.md#on)
- [once](ERC777SnapshotMock.md#once)
- [operatorBurn](ERC777SnapshotMock.md#operatorburn)
- [operatorBurn(address,uint256,bytes,bytes)](ERC777SnapshotMock.md#operatorburn(address,uint256,bytes,bytes))
- [operatorSend](ERC777SnapshotMock.md#operatorsend)
- [operatorSend(address,address,uint256,bytes,bytes)](ERC777SnapshotMock.md#operatorsend(address,address,uint256,bytes,bytes))
- [queryFilter](ERC777SnapshotMock.md#queryfilter)
- [removeAllListeners](ERC777SnapshotMock.md#removealllisteners)
- [removeListener](ERC777SnapshotMock.md#removelistener)
- [revokeOperator](ERC777SnapshotMock.md#revokeoperator)
- [revokeOperator(address)](ERC777SnapshotMock.md#revokeoperator(address))
- [send](ERC777SnapshotMock.md#send)
- [send(address,uint256,bytes)](ERC777SnapshotMock.md#send(address,uint256,bytes))
- [symbol](ERC777SnapshotMock.md#symbol)
- [symbol()](ERC777SnapshotMock.md#symbol())
- [totalSupply](ERC777SnapshotMock.md#totalsupply)
- [totalSupply()](ERC777SnapshotMock.md#totalsupply())
- [totalSupplyAt](ERC777SnapshotMock.md#totalsupplyat)
- [totalSupplyAt(uint128)](ERC777SnapshotMock.md#totalsupplyat(uint128))
- [totalSupplySnapshots](ERC777SnapshotMock.md#totalsupplysnapshots)
- [totalSupplySnapshots(uint256)](ERC777SnapshotMock.md#totalsupplysnapshots(uint256))
- [transfer](ERC777SnapshotMock.md#transfer)
- [transfer(address,uint256)](ERC777SnapshotMock.md#transfer(address,uint256))
- [transferFrom](ERC777SnapshotMock.md#transferfrom)
- [transferFrom(address,address,uint256)](ERC777SnapshotMock.md#transferfrom(address,address,uint256))
- [updateValueAtNowAccount](ERC777SnapshotMock.md#updatevalueatnowaccount)
- [updateValueAtNowAccount(address,uint256)](ERC777SnapshotMock.md#updatevalueatnowaccount(address,uint256))
- [getContractAddress](ERC777SnapshotMock.md#getcontractaddress)
- [getInterface](ERC777SnapshotMock.md#getinterface)
- [isIndexed](ERC777SnapshotMock.md#isindexed)

## Constructors

### constructor

• **new ERC777SnapshotMock**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |
| `contractInterface` | `ContractInterface` |
| `signerOrProvider?` | `Signer` \| `Provider` |

#### Inherited from

Contract.constructor

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:103

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
| `burn(address,uint256,bytes,bytes)` | (`account`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `burn(uint256,bytes)` | (`amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<`number`\> |
| `decimals()` | (`overrides?`: `CallOverrides`) => `Promise`<`number`\> |
| `defaultOperators` | (`overrides?`: `CallOverrides`) => `Promise`<`string`[]\> |
| `defaultOperators()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`[]\> |
| `getAccountValueAt` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getAccountValueAt(address,uint128)` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getTotalSupplyValueAt` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getTotalSupplyValueAt(uint128)` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `granularity` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `granularity()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isOperatorFor` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `isOperatorFor(address,address)` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `mint` | (`to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `mint(address,uint256,bytes,bytes)` | (`to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
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
| `updateValueAtNowAccount` | (`account`: `string`, `value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `updateValueAtNowAccount(address,uint256)` | (`account`: `string`, `value`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |

#### Overrides

Contract.callStatic

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:841

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
| `burn(address,uint256,bytes,bytes)` | (`account`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `burn(uint256,bytes)` | (`amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `decimals()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `defaultOperators` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `defaultOperators()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getAccountValueAt` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getAccountValueAt(address,uint128)` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getTotalSupplyValueAt` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getTotalSupplyValueAt(uint128)` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `granularity` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `granularity()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isOperatorFor` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isOperatorFor(address,address)` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `mint` | (`to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `mint(address,uint256,bytes,bytes)` | (`to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
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
| `updateValueAtNowAccount` | (`account`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `updateValueAtNowAccount(address,uint256)` | (`account`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

Contract.estimateGas

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:1209

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Approval` | (`owner`: `string`, `spender`: `string`, `value`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`], `Object`\> |
| `AuthorizedOperator` | (`operator`: `string`, `tokenHolder`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`], `Object`\> |
| `Burned` | (`operator`: `string`, `from`: `string`, `amount`: ``null``, `data`: ``null``, `operatorData`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`, `string`, `string`], `Object`\> |
| `Minted` | (`operator`: `string`, `to`: `string`, `amount`: ``null``, `data`: ``null``, `operatorData`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`, `string`, `string`], `Object`\> |
| `RevokedOperator` | (`operator`: `string`, `tokenHolder`: `string`) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`], `Object`\> |
| `Sent` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: ``null``, `data`: ``null``, `operatorData`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`, `BigNumber`, `string`, `string`], `Object`\> |
| `Transfer` | (`from`: `string`, `to`: `string`, `value`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `BigNumber`], `Object`\> |

#### Overrides

Contract.filters

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:1120

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
| `burn(address,uint256,bytes,bytes)` | (`account`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `burn(uint256,bytes)` | (`amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<[`number`]\> |
| `decimals()` | (`overrides?`: `CallOverrides`) => `Promise`<[`number`]\> |
| `defaultOperators` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`[]]\> |
| `defaultOperators()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`[]]\> |
| `getAccountValueAt` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `getAccountValueAt(address,uint128)` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `getTotalSupplyValueAt` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `getTotalSupplyValueAt(uint128)` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `granularity` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `granularity()` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `isOperatorFor` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `isOperatorFor(address,address)` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `mint` | (`to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `mint(address,uint256,bytes,bytes)` | (`to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
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
| `updateValueAtNowAccount` | (`account`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `updateValueAtNowAccount(address,uint256)` | (`account`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

Contract.functions

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:282

___

### interface

• **interface**: `ERC777SnapshotMockInterface`

#### Overrides

Contract.interface

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:280

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
| `burn(address,uint256,bytes,bytes)` | (`account`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `burn(uint256,bytes)` | (`amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `decimals` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `decimals()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `defaultOperators` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `defaultOperators()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getAccountValueAt` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getAccountValueAt(address,uint128)` | (`_owner`: `string`, `_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getTotalSupplyValueAt` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getTotalSupplyValueAt(uint128)` | (`_blockNumber`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `granularity` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `granularity()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `isOperatorFor` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `isOperatorFor(address,address)` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `mint` | (`to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `mint(address,uint256,bytes,bytes)` | (`to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
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
| `updateValueAtNowAccount` | (`account`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `updateValueAtNowAccount(address,uint256)` | (`account`: `string`, `value`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

Contract.populateTransaction

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:1483

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:564

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:572

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:580

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:586

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:592

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:598

___

### attach

▸ **attach**(`addressOrName`): [`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Overrides

Contract.attach

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:241

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:604

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:609

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:614

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:616

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:621

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:627

___

### burn(address,uint256,bytes,bytes)

▸ **burn(address,uint256,bytes,bytes)**(`account`, `amount`, `userData`, `operatorData`, `overrides?`): `Promise`<`ContractTransaction`\>

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:633

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:641

___

### connect

▸ **connect**(`signerOrProvider`): [`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Overrides

Contract.connect

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:240

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:647

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:649

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:651

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:653

___

### deployed

▸ **deployed**(): `Promise`<[`ERC777SnapshotMock`](ERC777SnapshotMock.md)\>

#### Returns

`Promise`<[`ERC777SnapshotMock`](ERC777SnapshotMock.md)\>

#### Overrides

Contract.deployed

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:242

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

### getAccountValueAt

▸ **getAccountValueAt**(`_owner`, `_blockNumber`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | `string` |
| `_blockNumber` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:655

___

### getAccountValueAt(address,uint128)

▸ **getAccountValueAt(address,uint128)**(`_owner`, `_blockNumber`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | `string` |
| `_blockNumber` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:661

___

### getTotalSupplyValueAt

▸ **getTotalSupplyValueAt**(`_blockNumber`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_blockNumber` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:667

___

### getTotalSupplyValueAt(uint128)

▸ **getTotalSupplyValueAt(uint128)**(`_blockNumber`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_blockNumber` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:672

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:677

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:679

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:681

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:687

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
| `eventFilter?` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\>[]

#### Overrides

Contract.listeners

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:244

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:267

___

### mint

▸ **mint**(`to`, `amount`, `userData`, `operatorData`, `overrides?`): `Promise`<`ContractTransaction`\>

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:693

___

### mint(address,uint256,bytes,bytes)

▸ **mint(address,uint256,bytes,bytes)**(`to`, `amount`, `userData`, `operatorData`, `overrides?`): `Promise`<`ContractTransaction`\>

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:701

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:709

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:711

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:247

▸ **off**(`eventName`, `listener`): [`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:268

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:251

▸ **on**(`eventName`, `listener`): [`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:269

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:255

▸ **once**(`eventName`, `listener`): [`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:270

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:713

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:721

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:729

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:738

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

Contract.queryFilter

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:274

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`ERC777SnapshotMock`](ERC777SnapshotMock.md)

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

[`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:263

▸ **removeAllListeners**(`eventName?`): [`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:272

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | extends `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<`EventArgsArray`, `EventArgsObject`\> |
| `listener` | [`TypedListener`](../modules.md#typedlistener)<`EventArgsArray`, `EventArgsObject`\> |

#### Returns

[`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:259

▸ **removeListener**(`eventName`, `listener`): [`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC777SnapshotMock`](ERC777SnapshotMock.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:271

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:747

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:752

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:757

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:764

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:771

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:773

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:775

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:777

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:779

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:784

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:789

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:796

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:803

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:809

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:815

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

packages/ethereum/types/ERC777SnapshotMock.d.ts:822

___

### updateValueAtNowAccount

▸ **updateValueAtNowAccount**(`account`, `value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:829

___

### updateValueAtNowAccount(address,uint256)

▸ **updateValueAtNowAccount(address,uint256)**(`account`, `value`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `value` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/ERC777SnapshotMock.d.ts:835

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
