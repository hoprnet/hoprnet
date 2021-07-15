[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprToken

# Class: HoprToken

## Hierarchy

- `Contract`

  ↳ **`HoprToken`**

## Table of contents

### Constructors

- [constructor](hoprtoken.md#constructor)

### Properties

- [\_deployedPromise](hoprtoken.md#_deployedpromise)
- [\_runningEvents](hoprtoken.md#_runningevents)
- [\_wrappedEmits](hoprtoken.md#_wrappedemits)
- [address](hoprtoken.md#address)
- [callStatic](hoprtoken.md#callstatic)
- [deployTransaction](hoprtoken.md#deploytransaction)
- [estimateGas](hoprtoken.md#estimategas)
- [filters](hoprtoken.md#filters)
- [functions](hoprtoken.md#functions)
- [interface](hoprtoken.md#interface)
- [populateTransaction](hoprtoken.md#populatetransaction)
- [provider](hoprtoken.md#provider)
- [resolvedAddress](hoprtoken.md#resolvedaddress)
- [signer](hoprtoken.md#signer)

### Methods

- [DEFAULT\_ADMIN\_ROLE](hoprtoken.md#default_admin_role)
- [DEFAULT\_ADMIN\_ROLE()](hoprtoken.md#default_admin_role())
- [MINTER\_ROLE](hoprtoken.md#minter_role)
- [MINTER\_ROLE()](hoprtoken.md#minter_role())
- [\_checkRunningEvents](hoprtoken.md#_checkrunningevents)
- [\_deployed](hoprtoken.md#_deployed)
- [\_wrapEvent](hoprtoken.md#_wrapevent)
- [accountSnapshots](hoprtoken.md#accountsnapshots)
- [accountSnapshots(address,uint256)](hoprtoken.md#accountsnapshots(address,uint256))
- [allowance](hoprtoken.md#allowance)
- [allowance(address,address)](hoprtoken.md#allowance(address,address))
- [approve](hoprtoken.md#approve)
- [approve(address,uint256)](hoprtoken.md#approve(address,uint256))
- [attach](hoprtoken.md#attach)
- [authorizeOperator](hoprtoken.md#authorizeoperator)
- [authorizeOperator(address)](hoprtoken.md#authorizeoperator(address))
- [balanceOf](hoprtoken.md#balanceof)
- [balanceOf(address)](hoprtoken.md#balanceof(address))
- [balanceOfAt](hoprtoken.md#balanceofat)
- [balanceOfAt(address,uint128)](hoprtoken.md#balanceofat(address,uint128))
- [burn](hoprtoken.md#burn)
- [burn(uint256,bytes)](hoprtoken.md#burn(uint256,bytes))
- [connect](hoprtoken.md#connect)
- [decimals](hoprtoken.md#decimals)
- [decimals()](hoprtoken.md#decimals())
- [defaultOperators](hoprtoken.md#defaultoperators)
- [defaultOperators()](hoprtoken.md#defaultoperators())
- [deployed](hoprtoken.md#deployed)
- [emit](hoprtoken.md#emit)
- [fallback](hoprtoken.md#fallback)
- [getRoleAdmin](hoprtoken.md#getroleadmin)
- [getRoleAdmin(bytes32)](hoprtoken.md#getroleadmin(bytes32))
- [getRoleMember](hoprtoken.md#getrolemember)
- [getRoleMember(bytes32,uint256)](hoprtoken.md#getrolemember(bytes32,uint256))
- [getRoleMemberCount](hoprtoken.md#getrolemembercount)
- [getRoleMemberCount(bytes32)](hoprtoken.md#getrolemembercount(bytes32))
- [grantRole](hoprtoken.md#grantrole)
- [grantRole(bytes32,address)](hoprtoken.md#grantrole(bytes32,address))
- [granularity](hoprtoken.md#granularity)
- [granularity()](hoprtoken.md#granularity())
- [hasRole](hoprtoken.md#hasrole)
- [hasRole(bytes32,address)](hoprtoken.md#hasrole(bytes32,address))
- [isOperatorFor](hoprtoken.md#isoperatorfor)
- [isOperatorFor(address,address)](hoprtoken.md#isoperatorfor(address,address))
- [listenerCount](hoprtoken.md#listenercount)
- [listeners](hoprtoken.md#listeners)
- [mint](hoprtoken.md#mint)
- [mint(address,uint256,bytes,bytes)](hoprtoken.md#mint(address,uint256,bytes,bytes))
- [name](hoprtoken.md#name)
- [name()](hoprtoken.md#name())
- [off](hoprtoken.md#off)
- [on](hoprtoken.md#on)
- [once](hoprtoken.md#once)
- [operatorBurn](hoprtoken.md#operatorburn)
- [operatorBurn(address,uint256,bytes,bytes)](hoprtoken.md#operatorburn(address,uint256,bytes,bytes))
- [operatorSend](hoprtoken.md#operatorsend)
- [operatorSend(address,address,uint256,bytes,bytes)](hoprtoken.md#operatorsend(address,address,uint256,bytes,bytes))
- [queryFilter](hoprtoken.md#queryfilter)
- [removeAllListeners](hoprtoken.md#removealllisteners)
- [removeListener](hoprtoken.md#removelistener)
- [renounceRole](hoprtoken.md#renouncerole)
- [renounceRole(bytes32,address)](hoprtoken.md#renouncerole(bytes32,address))
- [revokeOperator](hoprtoken.md#revokeoperator)
- [revokeOperator(address)](hoprtoken.md#revokeoperator(address))
- [revokeRole](hoprtoken.md#revokerole)
- [revokeRole(bytes32,address)](hoprtoken.md#revokerole(bytes32,address))
- [send](hoprtoken.md#send)
- [send(address,uint256,bytes)](hoprtoken.md#send(address,uint256,bytes))
- [symbol](hoprtoken.md#symbol)
- [symbol()](hoprtoken.md#symbol())
- [totalSupply](hoprtoken.md#totalsupply)
- [totalSupply()](hoprtoken.md#totalsupply())
- [totalSupplyAt](hoprtoken.md#totalsupplyat)
- [totalSupplyAt(uint128)](hoprtoken.md#totalsupplyat(uint128))
- [totalSupplySnapshots](hoprtoken.md#totalsupplysnapshots)
- [totalSupplySnapshots(uint256)](hoprtoken.md#totalsupplysnapshots(uint256))
- [transfer](hoprtoken.md#transfer)
- [transfer(address,uint256)](hoprtoken.md#transfer(address,uint256))
- [transferFrom](hoprtoken.md#transferfrom)
- [transferFrom(address,address,uint256)](hoprtoken.md#transferfrom(address,address,uint256))
- [getContractAddress](hoprtoken.md#getcontractaddress)
- [getInterface](hoprtoken.md#getinterface)
- [isIndexed](hoprtoken.md#isindexed)

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
| `DEFAULT_ADMIN_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `DEFAULT_ADMIN_ROLE()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `MINTER_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `MINTER_ROLE()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
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
| `getRoleAdmin` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `getRoleAdmin(bytes32)` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `getRoleMember` | (`role`: `BytesLike`, `index`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `getRoleMember(bytes32,uint256)` | (`role`: `BytesLike`, `index`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `getRoleMemberCount` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getRoleMemberCount(bytes32)` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `grantRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `grantRole(bytes32,address)` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `granularity` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `granularity()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `hasRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `hasRole(bytes32,address)` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `isOperatorFor` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `isOperatorFor(address,address)` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`boolean`\> |
| `mint` | (`account`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `mint(address,uint256,bytes,bytes)` | (`account`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `name()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `operatorBurn` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `operatorBurn(address,uint256,bytes,bytes)` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `operatorSend` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `operatorSend(address,address,uint256,bytes,bytes)` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `renounceRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `renounceRole(bytes32,address)` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `revokeOperator` | (`operator`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `revokeOperator(address)` | (`operator`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `revokeRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `revokeRole(bytes32,address)` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
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

packages/ethereum/types/HoprToken.d.ts:988

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
| `DEFAULT_ADMIN_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `DEFAULT_ADMIN_ROLE()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `MINTER_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `MINTER_ROLE()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
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
| `getRoleAdmin` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getRoleAdmin(bytes32)` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getRoleMember` | (`role`: `BytesLike`, `index`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getRoleMember(bytes32,uint256)` | (`role`: `BytesLike`, `index`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getRoleMemberCount` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getRoleMemberCount(bytes32)` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `grantRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `grantRole(bytes32,address)` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `granularity` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `granularity()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `hasRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `hasRole(bytes32,address)` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isOperatorFor` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `isOperatorFor(address,address)` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `mint` | (`account`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `mint(address,uint256,bytes,bytes)` | (`account`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `name()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `operatorBurn` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `operatorBurn(address,uint256,bytes,bytes)` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `operatorSend` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `operatorSend(address,address,uint256,bytes,bytes)` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `renounceRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `renounceRole(bytes32,address)` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `revokeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `revokeOperator(address)` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `revokeRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `revokeRole(bytes32,address)` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
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

packages/ethereum/types/HoprToken.d.ts:1423

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
| `RoleGranted` | (`role`: `BytesLike`, `account`: `string`, `sender`: `string`) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`, `string`], `Object`\> |
| `RoleRevoked` | (`role`: `BytesLike`, `account`: `string`, `sender`: `string`) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`, `string`], `Object`\> |
| `Sent` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: ``null``, `data`: ``null``, `operatorData`: ``null``) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`, `string`, `BigNumber`, `string`, `string`], `Object`\> |
| `Transfer` | (`from`: `string`, `to`: `string`, `value`: ``null``) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`, `BigNumber`], `Object`\> |

#### Overrides

Contract.filters

#### Defined in

packages/ethereum/types/HoprToken.d.ts:1316

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `DEFAULT_ADMIN_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `DEFAULT_ADMIN_ROLE()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `MINTER_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `MINTER_ROLE()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
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
| `getRoleAdmin` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getRoleAdmin(bytes32)` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getRoleMember` | (`role`: `BytesLike`, `index`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getRoleMember(bytes32,uint256)` | (`role`: `BytesLike`, `index`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getRoleMemberCount` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `getRoleMemberCount(bytes32)` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `grantRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `grantRole(bytes32,address)` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `granularity` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `granularity()` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `hasRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `hasRole(bytes32,address)` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `isOperatorFor` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `isOperatorFor(address,address)` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `mint` | (`account`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `mint(address,uint256,bytes,bytes)` | (`account`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `name()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `operatorBurn` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `operatorBurn(address,uint256,bytes,bytes)` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `operatorSend` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `operatorSend(address,address,uint256,bytes,bytes)` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `renounceRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `renounceRole(bytes32,address)` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `revokeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `revokeOperator(address)` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `revokeRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `revokeRole(bytes32,address)` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
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

packages/ethereum/types/HoprToken.d.ts:331

___

### interface

• **interface**: `HoprTokenInterface`

#### Overrides

Contract.interface

#### Defined in

packages/ethereum/types/HoprToken.d.ts:329

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `DEFAULT_ADMIN_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `DEFAULT_ADMIN_ROLE()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `MINTER_ROLE` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `MINTER_ROLE()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
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
| `getRoleAdmin` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getRoleAdmin(bytes32)` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getRoleMember` | (`role`: `BytesLike`, `index`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getRoleMember(bytes32,uint256)` | (`role`: `BytesLike`, `index`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getRoleMemberCount` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getRoleMemberCount(bytes32)` | (`role`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `grantRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `grantRole(bytes32,address)` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `granularity` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `granularity()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `hasRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `hasRole(bytes32,address)` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `isOperatorFor` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `isOperatorFor(address,address)` | (`operator`: `string`, `tokenHolder`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `mint` | (`account`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `mint(address,uint256,bytes,bytes)` | (`account`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `name` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `name()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `operatorBurn` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `operatorBurn(address,uint256,bytes,bytes)` | (`account`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `operatorSend` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `operatorSend(address,address,uint256,bytes,bytes)` | (`sender`: `string`, `recipient`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `renounceRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `renounceRole(bytes32,address)` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `revokeOperator` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `revokeOperator(address)` | (`operator`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `revokeRole` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `revokeRole(bytes32,address)` | (`role`: `BytesLike`, `account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
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

packages/ethereum/types/HoprToken.d.ts:1749

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

### DEFAULT\_ADMIN\_ROLE

▸ **DEFAULT_ADMIN_ROLE**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprToken.d.ts:662

___

### DEFAULT\_ADMIN\_ROLE()

▸ **DEFAULT_ADMIN_ROLE()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprToken.d.ts:662

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

packages/ethereum/types/HoprToken.d.ts:666

___

### MINTER\_ROLE()

▸ **MINTER_ROLE()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprToken.d.ts:666

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

packages/ethereum/types/HoprToken.d.ts:670

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

packages/ethereum/types/HoprToken.d.ts:676

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

packages/ethereum/types/HoprToken.d.ts:686

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

packages/ethereum/types/HoprToken.d.ts:690

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

packages/ethereum/types/HoprToken.d.ts:698

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

packages/ethereum/types/HoprToken.d.ts:702

___

### attach

▸ **attach**(`addressOrName`): [`HoprToken`](hoprtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`HoprToken`](hoprtoken.md)

#### Overrides

Contract.attach

#### Defined in

packages/ethereum/types/HoprToken.d.ts:290

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

packages/ethereum/types/HoprToken.d.ts:710

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

packages/ethereum/types/HoprToken.d.ts:713

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

packages/ethereum/types/HoprToken.d.ts:720

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

packages/ethereum/types/HoprToken.d.ts:720

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

packages/ethereum/types/HoprToken.d.ts:727

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

packages/ethereum/types/HoprToken.d.ts:731

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

packages/ethereum/types/HoprToken.d.ts:739

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

packages/ethereum/types/HoprToken.d.ts:743

___

### connect

▸ **connect**(`signerOrProvider`): [`HoprToken`](hoprtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`HoprToken`](hoprtoken.md)

#### Overrides

Contract.connect

#### Defined in

packages/ethereum/types/HoprToken.d.ts:289

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

packages/ethereum/types/HoprToken.d.ts:751

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

packages/ethereum/types/HoprToken.d.ts:751

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

packages/ethereum/types/HoprToken.d.ts:755

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

packages/ethereum/types/HoprToken.d.ts:755

___

### deployed

▸ **deployed**(): `Promise`<[`HoprToken`](hoprtoken.md)\>

#### Returns

`Promise`<[`HoprToken`](hoprtoken.md)\>

#### Overrides

Contract.deployed

#### Defined in

packages/ethereum/types/HoprToken.d.ts:291

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

packages/ethereum/types/HoprToken.d.ts:759

___

### getRoleAdmin(bytes32)

▸ **getRoleAdmin(bytes32)**(`role`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprToken.d.ts:759

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

packages/ethereum/types/HoprToken.d.ts:766

___

### getRoleMember(bytes32,uint256)

▸ **getRoleMember(bytes32,uint256)**(`role`, `index`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | `BytesLike` |
| `index` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprToken.d.ts:770

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

packages/ethereum/types/HoprToken.d.ts:778

___

### getRoleMemberCount(bytes32)

▸ **getRoleMemberCount(bytes32)**(`role`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/HoprToken.d.ts:781

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

packages/ethereum/types/HoprToken.d.ts:788

___

### grantRole(bytes32,address)

▸ **grantRole(bytes32,address)**(`role`, `account`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | `BytesLike` |
| `account` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprToken.d.ts:792

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

packages/ethereum/types/HoprToken.d.ts:800

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

packages/ethereum/types/HoprToken.d.ts:800

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

packages/ethereum/types/HoprToken.d.ts:804

___

### hasRole(bytes32,address)

▸ **hasRole(bytes32,address)**(`role`, `account`, `overrides?`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | `BytesLike` |
| `account` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

packages/ethereum/types/HoprToken.d.ts:808

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

packages/ethereum/types/HoprToken.d.ts:816

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

packages/ethereum/types/HoprToken.d.ts:820

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

packages/ethereum/types/HoprToken.d.ts:293

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

packages/ethereum/types/HoprToken.d.ts:316

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

packages/ethereum/types/HoprToken.d.ts:828

___

### mint(address,uint256,bytes,bytes)

▸ **mint(address,uint256,bytes,bytes)**(`account`, `amount`, `userData`, `operatorData`, `overrides?`): `Promise`<`ContractTransaction`\>

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

packages/ethereum/types/HoprToken.d.ts:834

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

packages/ethereum/types/HoprToken.d.ts:844

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

packages/ethereum/types/HoprToken.d.ts:844

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprToken`](hoprtoken.md)

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

[`HoprToken`](hoprtoken.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/HoprToken.d.ts:296

▸ **off**(`eventName`, `listener`): [`HoprToken`](hoprtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprToken`](hoprtoken.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/HoprToken.d.ts:317

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprToken`](hoprtoken.md)

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

[`HoprToken`](hoprtoken.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/HoprToken.d.ts:300

▸ **on**(`eventName`, `listener`): [`HoprToken`](hoprtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprToken`](hoprtoken.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/HoprToken.d.ts:318

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprToken`](hoprtoken.md)

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

[`HoprToken`](hoprtoken.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/HoprToken.d.ts:304

▸ **once**(`eventName`, `listener`): [`HoprToken`](hoprtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprToken`](hoprtoken.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/HoprToken.d.ts:319

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

packages/ethereum/types/HoprToken.d.ts:848

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

packages/ethereum/types/HoprToken.d.ts:854

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

packages/ethereum/types/HoprToken.d.ts:864

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

packages/ethereum/types/HoprToken.d.ts:871

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

packages/ethereum/types/HoprToken.d.ts:323

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`HoprToken`](hoprtoken.md)

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

[`HoprToken`](hoprtoken.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/HoprToken.d.ts:312

▸ **removeAllListeners**(`eventName?`): [`HoprToken`](hoprtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`HoprToken`](hoprtoken.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/HoprToken.d.ts:321

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprToken`](hoprtoken.md)

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

[`HoprToken`](hoprtoken.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/HoprToken.d.ts:308

▸ **removeListener**(`eventName`, `listener`): [`HoprToken`](hoprtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprToken`](hoprtoken.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/HoprToken.d.ts:320

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

packages/ethereum/types/HoprToken.d.ts:882

___

### renounceRole(bytes32,address)

▸ **renounceRole(bytes32,address)**(`role`, `account`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | `BytesLike` |
| `account` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprToken.d.ts:886

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

packages/ethereum/types/HoprToken.d.ts:894

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

packages/ethereum/types/HoprToken.d.ts:897

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

packages/ethereum/types/HoprToken.d.ts:904

___

### revokeRole(bytes32,address)

▸ **revokeRole(bytes32,address)**(`role`, `account`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | `BytesLike` |
| `account` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprToken.d.ts:908

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

packages/ethereum/types/HoprToken.d.ts:916

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

packages/ethereum/types/HoprToken.d.ts:921

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

packages/ethereum/types/HoprToken.d.ts:930

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

packages/ethereum/types/HoprToken.d.ts:930

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

packages/ethereum/types/HoprToken.d.ts:934

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

packages/ethereum/types/HoprToken.d.ts:934

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

packages/ethereum/types/HoprToken.d.ts:938

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

packages/ethereum/types/HoprToken.d.ts:941

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

packages/ethereum/types/HoprToken.d.ts:948

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

packages/ethereum/types/HoprToken.d.ts:953

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

packages/ethereum/types/HoprToken.d.ts:962

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

packages/ethereum/types/HoprToken.d.ts:966

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

packages/ethereum/types/HoprToken.d.ts:974

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

packages/ethereum/types/HoprToken.d.ts:979

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
