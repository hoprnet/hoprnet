[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprToken](../modules/contracts_hoprtoken.md) / HoprToken

# Class: HoprToken

[contracts/HoprToken](../modules/contracts_hoprtoken.md).HoprToken

## Hierarchy

- *Contract*

  ↳ **HoprToken**

## Table of contents

### Constructors

- [constructor](contracts_hoprtoken.hoprtoken.md#constructor)

### Properties

- [\_deployedPromise](contracts_hoprtoken.hoprtoken.md#_deployedpromise)
- [\_runningEvents](contracts_hoprtoken.hoprtoken.md#_runningevents)
- [\_wrappedEmits](contracts_hoprtoken.hoprtoken.md#_wrappedemits)
- [address](contracts_hoprtoken.hoprtoken.md#address)
- [callStatic](contracts_hoprtoken.hoprtoken.md#callstatic)
- [deployTransaction](contracts_hoprtoken.hoprtoken.md#deploytransaction)
- [estimateGas](contracts_hoprtoken.hoprtoken.md#estimategas)
- [filters](contracts_hoprtoken.hoprtoken.md#filters)
- [functions](contracts_hoprtoken.hoprtoken.md#functions)
- [interface](contracts_hoprtoken.hoprtoken.md#interface)
- [populateTransaction](contracts_hoprtoken.hoprtoken.md#populatetransaction)
- [provider](contracts_hoprtoken.hoprtoken.md#provider)
- [resolvedAddress](contracts_hoprtoken.hoprtoken.md#resolvedaddress)
- [signer](contracts_hoprtoken.hoprtoken.md#signer)

### Methods

- [DEFAULT\_ADMIN\_ROLE](contracts_hoprtoken.hoprtoken.md#default_admin_role)
- [DEFAULT\_ADMIN\_ROLE()](contracts_hoprtoken.hoprtoken.md#default_admin_role())
- [MINTER\_ROLE](contracts_hoprtoken.hoprtoken.md#minter_role)
- [MINTER\_ROLE()](contracts_hoprtoken.hoprtoken.md#minter_role())
- [\_checkRunningEvents](contracts_hoprtoken.hoprtoken.md#_checkrunningevents)
- [\_deployed](contracts_hoprtoken.hoprtoken.md#_deployed)
- [\_wrapEvent](contracts_hoprtoken.hoprtoken.md#_wrapevent)
- [accountSnapshots](contracts_hoprtoken.hoprtoken.md#accountsnapshots)
- [accountSnapshots(address,uint256)](contracts_hoprtoken.hoprtoken.md#accountsnapshots(address,uint256))
- [allowance](contracts_hoprtoken.hoprtoken.md#allowance)
- [allowance(address,address)](contracts_hoprtoken.hoprtoken.md#allowance(address,address))
- [approve](contracts_hoprtoken.hoprtoken.md#approve)
- [approve(address,uint256)](contracts_hoprtoken.hoprtoken.md#approve(address,uint256))
- [attach](contracts_hoprtoken.hoprtoken.md#attach)
- [authorizeOperator](contracts_hoprtoken.hoprtoken.md#authorizeoperator)
- [authorizeOperator(address)](contracts_hoprtoken.hoprtoken.md#authorizeoperator(address))
- [balanceOf](contracts_hoprtoken.hoprtoken.md#balanceof)
- [balanceOf(address)](contracts_hoprtoken.hoprtoken.md#balanceof(address))
- [balanceOfAt](contracts_hoprtoken.hoprtoken.md#balanceofat)
- [balanceOfAt(address,uint128)](contracts_hoprtoken.hoprtoken.md#balanceofat(address,uint128))
- [burn](contracts_hoprtoken.hoprtoken.md#burn)
- [burn(uint256,bytes)](contracts_hoprtoken.hoprtoken.md#burn(uint256,bytes))
- [connect](contracts_hoprtoken.hoprtoken.md#connect)
- [decimals](contracts_hoprtoken.hoprtoken.md#decimals)
- [decimals()](contracts_hoprtoken.hoprtoken.md#decimals())
- [defaultOperators](contracts_hoprtoken.hoprtoken.md#defaultoperators)
- [defaultOperators()](contracts_hoprtoken.hoprtoken.md#defaultoperators())
- [deployed](contracts_hoprtoken.hoprtoken.md#deployed)
- [emit](contracts_hoprtoken.hoprtoken.md#emit)
- [fallback](contracts_hoprtoken.hoprtoken.md#fallback)
- [getRoleAdmin](contracts_hoprtoken.hoprtoken.md#getroleadmin)
- [getRoleAdmin(bytes32)](contracts_hoprtoken.hoprtoken.md#getroleadmin(bytes32))
- [getRoleMember](contracts_hoprtoken.hoprtoken.md#getrolemember)
- [getRoleMember(bytes32,uint256)](contracts_hoprtoken.hoprtoken.md#getrolemember(bytes32,uint256))
- [getRoleMemberCount](contracts_hoprtoken.hoprtoken.md#getrolemembercount)
- [getRoleMemberCount(bytes32)](contracts_hoprtoken.hoprtoken.md#getrolemembercount(bytes32))
- [grantRole](contracts_hoprtoken.hoprtoken.md#grantrole)
- [grantRole(bytes32,address)](contracts_hoprtoken.hoprtoken.md#grantrole(bytes32,address))
- [granularity](contracts_hoprtoken.hoprtoken.md#granularity)
- [granularity()](contracts_hoprtoken.hoprtoken.md#granularity())
- [hasRole](contracts_hoprtoken.hoprtoken.md#hasrole)
- [hasRole(bytes32,address)](contracts_hoprtoken.hoprtoken.md#hasrole(bytes32,address))
- [isOperatorFor](contracts_hoprtoken.hoprtoken.md#isoperatorfor)
- [isOperatorFor(address,address)](contracts_hoprtoken.hoprtoken.md#isoperatorfor(address,address))
- [listenerCount](contracts_hoprtoken.hoprtoken.md#listenercount)
- [listeners](contracts_hoprtoken.hoprtoken.md#listeners)
- [mint](contracts_hoprtoken.hoprtoken.md#mint)
- [mint(address,uint256,bytes,bytes)](contracts_hoprtoken.hoprtoken.md#mint(address,uint256,bytes,bytes))
- [name](contracts_hoprtoken.hoprtoken.md#name)
- [name()](contracts_hoprtoken.hoprtoken.md#name())
- [off](contracts_hoprtoken.hoprtoken.md#off)
- [on](contracts_hoprtoken.hoprtoken.md#on)
- [once](contracts_hoprtoken.hoprtoken.md#once)
- [operatorBurn](contracts_hoprtoken.hoprtoken.md#operatorburn)
- [operatorBurn(address,uint256,bytes,bytes)](contracts_hoprtoken.hoprtoken.md#operatorburn(address,uint256,bytes,bytes))
- [operatorSend](contracts_hoprtoken.hoprtoken.md#operatorsend)
- [operatorSend(address,address,uint256,bytes,bytes)](contracts_hoprtoken.hoprtoken.md#operatorsend(address,address,uint256,bytes,bytes))
- [queryFilter](contracts_hoprtoken.hoprtoken.md#queryfilter)
- [removeAllListeners](contracts_hoprtoken.hoprtoken.md#removealllisteners)
- [removeListener](contracts_hoprtoken.hoprtoken.md#removelistener)
- [renounceRole](contracts_hoprtoken.hoprtoken.md#renouncerole)
- [renounceRole(bytes32,address)](contracts_hoprtoken.hoprtoken.md#renouncerole(bytes32,address))
- [revokeOperator](contracts_hoprtoken.hoprtoken.md#revokeoperator)
- [revokeOperator(address)](contracts_hoprtoken.hoprtoken.md#revokeoperator(address))
- [revokeRole](contracts_hoprtoken.hoprtoken.md#revokerole)
- [revokeRole(bytes32,address)](contracts_hoprtoken.hoprtoken.md#revokerole(bytes32,address))
- [send](contracts_hoprtoken.hoprtoken.md#send)
- [send(address,uint256,bytes)](contracts_hoprtoken.hoprtoken.md#send(address,uint256,bytes))
- [symbol](contracts_hoprtoken.hoprtoken.md#symbol)
- [symbol()](contracts_hoprtoken.hoprtoken.md#symbol())
- [totalSupply](contracts_hoprtoken.hoprtoken.md#totalsupply)
- [totalSupply()](contracts_hoprtoken.hoprtoken.md#totalsupply())
- [totalSupplyAt](contracts_hoprtoken.hoprtoken.md#totalsupplyat)
- [totalSupplyAt(uint128)](contracts_hoprtoken.hoprtoken.md#totalsupplyat(uint128))
- [totalSupplySnapshots](contracts_hoprtoken.hoprtoken.md#totalsupplysnapshots)
- [totalSupplySnapshots(uint256)](contracts_hoprtoken.hoprtoken.md#totalsupplysnapshots(uint256))
- [transfer](contracts_hoprtoken.hoprtoken.md#transfer)
- [transfer(address,uint256)](contracts_hoprtoken.hoprtoken.md#transfer(address,uint256))
- [transferFrom](contracts_hoprtoken.hoprtoken.md#transferfrom)
- [transferFrom(address,address,uint256)](contracts_hoprtoken.hoprtoken.md#transferfrom(address,address,uint256))
- [getContractAddress](contracts_hoprtoken.hoprtoken.md#getcontractaddress)
- [getInterface](contracts_hoprtoken.hoprtoken.md#getinterface)
- [isIndexed](contracts_hoprtoken.hoprtoken.md#isindexed)

## Constructors

### constructor

\+ **new HoprToken**(`addressOrName`: *string*, `contractInterface`: ContractInterface, `signerOrProvider?`: *Provider* \| *Signer*): [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |
| `contractInterface` | ContractInterface |
| `signerOrProvider?` | *Provider* \| *Signer* |

**Returns:** [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

Inherited from: Contract.constructor

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:98

## Properties

### \_deployedPromise

• **\_deployedPromise**: *Promise*<Contract\>

Inherited from: Contract.\_deployedPromise

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:92

___

### \_runningEvents

• **\_runningEvents**: *object*

#### Type declaration

Inherited from: Contract.\_runningEvents

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:93

___

### \_wrappedEmits

• **\_wrappedEmits**: *object*

#### Type declaration

Inherited from: Contract.\_wrappedEmits

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:96

___

### address

• `Readonly` **address**: *string*

Inherited from: Contract.address

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:71

___

### callStatic

• **callStatic**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `DEFAULT_ADMIN_ROLE` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `DEFAULT_ADMIN_ROLE()` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `MINTER_ROLE` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `MINTER_ROLE()` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `accountSnapshots` | (`arg0`: *string*, `arg1`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*, *BigNumber*] & { `fromBlock`: *BigNumber* ; `value`: *BigNumber*  }\> |
| `accountSnapshots(address,uint256)` | (`arg0`: *string*, `arg1`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*, *BigNumber*] & { `fromBlock`: *BigNumber* ; `value`: *BigNumber*  }\> |
| `allowance` | (`holder`: *string*, `spender`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `allowance(address,address)` | (`holder`: *string*, `spender`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `approve` | (`spender`: *string*, `value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `approve(address,uint256)` | (`spender`: *string*, `value`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `authorizeOperator` | (`operator`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `authorizeOperator(address)` | (`operator`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `balanceOf` | (`tokenHolder`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `balanceOf(address)` | (`tokenHolder`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `balanceOfAt` | (`_owner`: *string*, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `balanceOfAt(address,uint128)` | (`_owner`: *string*, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `burn` | (`amount`: BigNumberish, `data`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `burn(uint256,bytes)` | (`amount`: BigNumberish, `data`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `decimals` | (`overrides?`: CallOverrides) => *Promise*<number\> |
| `decimals()` | (`overrides?`: CallOverrides) => *Promise*<number\> |
| `defaultOperators` | (`overrides?`: CallOverrides) => *Promise*<string[]\> |
| `defaultOperators()` | (`overrides?`: CallOverrides) => *Promise*<string[]\> |
| `getRoleAdmin` | (`role`: BytesLike, `overrides?`: CallOverrides) => *Promise*<string\> |
| `getRoleAdmin(bytes32)` | (`role`: BytesLike, `overrides?`: CallOverrides) => *Promise*<string\> |
| `getRoleMember` | (`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<string\> |
| `getRoleMember(bytes32,uint256)` | (`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<string\> |
| `getRoleMemberCount` | (`role`: BytesLike, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `getRoleMemberCount(bytes32)` | (`role`: BytesLike, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `grantRole` | (`role`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `grantRole(bytes32,address)` | (`role`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `granularity` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `granularity()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `hasRole` | (`role`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `hasRole(bytes32,address)` | (`role`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `isOperatorFor` | (`operator`: *string*, `tokenHolder`: *string*, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `isOperatorFor(address,address)` | (`operator`: *string*, `tokenHolder`: *string*, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `mint` | (`account`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `mint(address,uint256,bytes,bytes)` | (`account`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `name` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `name()` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `operatorBurn` | (`account`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `operatorBurn(address,uint256,bytes,bytes)` | (`account`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `operatorSend` | (`sender`: *string*, `recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `operatorSend(address,address,uint256,bytes,bytes)` | (`sender`: *string*, `recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `renounceRole` | (`role`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `renounceRole(bytes32,address)` | (`role`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `revokeOperator` | (`operator`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `revokeOperator(address)` | (`operator`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `revokeRole` | (`role`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `revokeRole(bytes32,address)` | (`role`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `send` | (`recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `send(address,uint256,bytes)` | (`recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `symbol` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `symbol()` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupplyAt` | (`_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupplyAt(uint128)` | (`_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupplySnapshots` | (`arg0`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*, *BigNumber*] & { `fromBlock`: *BigNumber* ; `value`: *BigNumber*  }\> |
| `totalSupplySnapshots(uint256)` | (`arg0`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*, *BigNumber*] & { `fromBlock`: *BigNumber* ; `value`: *BigNumber*  }\> |
| `transfer` | (`recipient`: *string*, `amount`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `transfer(address,uint256)` | (`recipient`: *string*, `amount`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `transferFrom` | (`holder`: *string*, `recipient`: *string*, `amount`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |
| `transferFrom(address,address,uint256)` | (`holder`: *string*, `recipient`: *string*, `amount`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<boolean\> |

Overrides: Contract.callStatic

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:988

___

### deployTransaction

• `Readonly` **deployTransaction**: TransactionResponse

Inherited from: Contract.deployTransaction

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:91

___

### estimateGas

• **estimateGas**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `DEFAULT_ADMIN_ROLE` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `DEFAULT_ADMIN_ROLE()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `MINTER_ROLE` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `MINTER_ROLE()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `accountSnapshots` | (`arg0`: *string*, `arg1`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `accountSnapshots(address,uint256)` | (`arg0`: *string*, `arg1`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `allowance` | (`holder`: *string*, `spender`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `allowance(address,address)` | (`holder`: *string*, `spender`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `approve` | (`spender`: *string*, `value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `approve(address,uint256)` | (`spender`: *string*, `value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `authorizeOperator` | (`operator`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `authorizeOperator(address)` | (`operator`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `balanceOf` | (`tokenHolder`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `balanceOf(address)` | (`tokenHolder`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `balanceOfAt` | (`_owner`: *string*, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `balanceOfAt(address,uint128)` | (`_owner`: *string*, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `burn` | (`amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `burn(uint256,bytes)` | (`amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `decimals` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `decimals()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `defaultOperators` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `defaultOperators()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `getRoleAdmin` | (`role`: BytesLike, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `getRoleAdmin(bytes32)` | (`role`: BytesLike, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `getRoleMember` | (`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `getRoleMember(bytes32,uint256)` | (`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `getRoleMemberCount` | (`role`: BytesLike, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `getRoleMemberCount(bytes32)` | (`role`: BytesLike, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `grantRole` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `grantRole(bytes32,address)` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `granularity` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `granularity()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `hasRole` | (`role`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `hasRole(bytes32,address)` | (`role`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `isOperatorFor` | (`operator`: *string*, `tokenHolder`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `isOperatorFor(address,address)` | (`operator`: *string*, `tokenHolder`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `mint` | (`account`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `mint(address,uint256,bytes,bytes)` | (`account`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `name` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `name()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `operatorBurn` | (`account`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `operatorBurn(address,uint256,bytes,bytes)` | (`account`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `operatorSend` | (`sender`: *string*, `recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `operatorSend(address,address,uint256,bytes,bytes)` | (`sender`: *string*, `recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `renounceRole` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `renounceRole(bytes32,address)` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `revokeOperator` | (`operator`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `revokeOperator(address)` | (`operator`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `revokeRole` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `revokeRole(bytes32,address)` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `send` | (`recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `send(address,uint256,bytes)` | (`recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `symbol` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `symbol()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupplyAt` | (`_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupplyAt(uint128)` | (`_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupplySnapshots` | (`arg0`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `totalSupplySnapshots(uint256)` | (`arg0`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `transfer` | (`recipient`: *string*, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transfer(address,uint256)` | (`recipient`: *string*, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transferFrom` | (`holder`: *string*, `recipient`: *string*, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `transferFrom(address,address,uint256)` | (`holder`: *string*, `recipient`: *string*, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |

Overrides: Contract.estimateGas

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:1423

___

### filters

• **filters**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Approval` | (`owner`: *string*, `spender`: *string*, `value`: ``null``) => [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*, *BigNumber*], { `owner`: *string* ; `spender`: *string* ; `value`: *BigNumber*  }\> |
| `AuthorizedOperator` | (`operator`: *string*, `tokenHolder`: *string*) => [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*], { `operator`: *string* ; `tokenHolder`: *string*  }\> |
| `Burned` | (`operator`: *string*, `from`: *string*, `amount`: ``null``, `data`: ``null``, `operatorData`: ``null``) => [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*, *BigNumber*, *string*, *string*], { `amount`: *BigNumber* ; `data`: *string* ; `from`: *string* ; `operator`: *string* ; `operatorData`: *string*  }\> |
| `Minted` | (`operator`: *string*, `to`: *string*, `amount`: ``null``, `data`: ``null``, `operatorData`: ``null``) => [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*, *BigNumber*, *string*, *string*], { `amount`: *BigNumber* ; `data`: *string* ; `operator`: *string* ; `operatorData`: *string* ; `to`: *string*  }\> |
| `RevokedOperator` | (`operator`: *string*, `tokenHolder`: *string*) => [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*], { `operator`: *string* ; `tokenHolder`: *string*  }\> |
| `RoleGranted` | (`role`: BytesLike, `account`: *string*, `sender`: *string*) => [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*, *string*], { `account`: *string* ; `role`: *string* ; `sender`: *string*  }\> |
| `RoleRevoked` | (`role`: BytesLike, `account`: *string*, `sender`: *string*) => [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*, *string*], { `account`: *string* ; `role`: *string* ; `sender`: *string*  }\> |
| `Sent` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: ``null``, `data`: ``null``, `operatorData`: ``null``) => [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*, *string*, *BigNumber*, *string*, *string*], { `amount`: *BigNumber* ; `data`: *string* ; `from`: *string* ; `operator`: *string* ; `operatorData`: *string* ; `to`: *string*  }\> |
| `Transfer` | (`from`: *string*, `to`: *string*, `value`: ``null``) => [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*, *BigNumber*], { `from`: *string* ; `to`: *string* ; `value`: *BigNumber*  }\> |

Overrides: Contract.filters

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:1316

___

### functions

• **functions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `DEFAULT_ADMIN_ROLE` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `DEFAULT_ADMIN_ROLE()` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `MINTER_ROLE` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `MINTER_ROLE()` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `accountSnapshots` | (`arg0`: *string*, `arg1`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*, *BigNumber*] & { `fromBlock`: *BigNumber* ; `value`: *BigNumber*  }\> |
| `accountSnapshots(address,uint256)` | (`arg0`: *string*, `arg1`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*, *BigNumber*] & { `fromBlock`: *BigNumber* ; `value`: *BigNumber*  }\> |
| `allowance` | (`holder`: *string*, `spender`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `allowance(address,address)` | (`holder`: *string*, `spender`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `approve` | (`spender`: *string*, `value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `approve(address,uint256)` | (`spender`: *string*, `value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `authorizeOperator` | (`operator`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `authorizeOperator(address)` | (`operator`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `balanceOf` | (`tokenHolder`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `balanceOf(address)` | (`tokenHolder`: *string*, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `balanceOfAt` | (`_owner`: *string*, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `balanceOfAt(address,uint128)` | (`_owner`: *string*, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `burn` | (`amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `burn(uint256,bytes)` | (`amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `decimals` | (`overrides?`: CallOverrides) => *Promise*<[*number*]\> |
| `decimals()` | (`overrides?`: CallOverrides) => *Promise*<[*number*]\> |
| `defaultOperators` | (`overrides?`: CallOverrides) => *Promise*<[*string*[]]\> |
| `defaultOperators()` | (`overrides?`: CallOverrides) => *Promise*<[*string*[]]\> |
| `getRoleAdmin` | (`role`: BytesLike, `overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `getRoleAdmin(bytes32)` | (`role`: BytesLike, `overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `getRoleMember` | (`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `getRoleMember(bytes32,uint256)` | (`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `getRoleMemberCount` | (`role`: BytesLike, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `getRoleMemberCount(bytes32)` | (`role`: BytesLike, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `grantRole` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `grantRole(bytes32,address)` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `granularity` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `granularity()` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `hasRole` | (`role`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<[*boolean*]\> |
| `hasRole(bytes32,address)` | (`role`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<[*boolean*]\> |
| `isOperatorFor` | (`operator`: *string*, `tokenHolder`: *string*, `overrides?`: CallOverrides) => *Promise*<[*boolean*]\> |
| `isOperatorFor(address,address)` | (`operator`: *string*, `tokenHolder`: *string*, `overrides?`: CallOverrides) => *Promise*<[*boolean*]\> |
| `mint` | (`account`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `mint(address,uint256,bytes,bytes)` | (`account`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `name` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `name()` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `operatorBurn` | (`account`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `operatorBurn(address,uint256,bytes,bytes)` | (`account`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `operatorSend` | (`sender`: *string*, `recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `operatorSend(address,address,uint256,bytes,bytes)` | (`sender`: *string*, `recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `renounceRole` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `renounceRole(bytes32,address)` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `revokeOperator` | (`operator`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `revokeOperator(address)` | (`operator`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `revokeRole` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `revokeRole(bytes32,address)` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `send` | (`recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `send(address,uint256,bytes)` | (`recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `symbol` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `symbol()` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `totalSupplyAt` | (`_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `totalSupplyAt(uint128)` | (`_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `totalSupplySnapshots` | (`arg0`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*, *BigNumber*] & { `fromBlock`: *BigNumber* ; `value`: *BigNumber*  }\> |
| `totalSupplySnapshots(uint256)` | (`arg0`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*, *BigNumber*] & { `fromBlock`: *BigNumber* ; `value`: *BigNumber*  }\> |
| `transfer` | (`recipient`: *string*, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transfer(address,uint256)` | (`recipient`: *string*, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transferFrom` | (`holder`: *string*, `recipient`: *string*, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `transferFrom(address,address,uint256)` | (`holder`: *string*, `recipient`: *string*, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |

Overrides: Contract.functions

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:331

___

### interface

• **interface**: [*HoprTokenInterface*](../interfaces/contracts_hoprtoken.hoprtokeninterface.md)

Overrides: Contract.interface

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:329

___

### populateTransaction

• **populateTransaction**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `DEFAULT_ADMIN_ROLE` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `DEFAULT_ADMIN_ROLE()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `MINTER_ROLE` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `MINTER_ROLE()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `accountSnapshots` | (`arg0`: *string*, `arg1`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `accountSnapshots(address,uint256)` | (`arg0`: *string*, `arg1`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `allowance` | (`holder`: *string*, `spender`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `allowance(address,address)` | (`holder`: *string*, `spender`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `approve` | (`spender`: *string*, `value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `approve(address,uint256)` | (`spender`: *string*, `value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `authorizeOperator` | (`operator`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `authorizeOperator(address)` | (`operator`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `balanceOf` | (`tokenHolder`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `balanceOf(address)` | (`tokenHolder`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `balanceOfAt` | (`_owner`: *string*, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `balanceOfAt(address,uint128)` | (`_owner`: *string*, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `burn` | (`amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `burn(uint256,bytes)` | (`amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `decimals` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `decimals()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `defaultOperators` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `defaultOperators()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `getRoleAdmin` | (`role`: BytesLike, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `getRoleAdmin(bytes32)` | (`role`: BytesLike, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `getRoleMember` | (`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `getRoleMember(bytes32,uint256)` | (`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `getRoleMemberCount` | (`role`: BytesLike, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `getRoleMemberCount(bytes32)` | (`role`: BytesLike, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `grantRole` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `grantRole(bytes32,address)` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `granularity` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `granularity()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `hasRole` | (`role`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `hasRole(bytes32,address)` | (`role`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `isOperatorFor` | (`operator`: *string*, `tokenHolder`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `isOperatorFor(address,address)` | (`operator`: *string*, `tokenHolder`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `mint` | (`account`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `mint(address,uint256,bytes,bytes)` | (`account`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `name` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `name()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `operatorBurn` | (`account`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `operatorBurn(address,uint256,bytes,bytes)` | (`account`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `operatorSend` | (`sender`: *string*, `recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `operatorSend(address,address,uint256,bytes,bytes)` | (`sender`: *string*, `recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `renounceRole` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `renounceRole(bytes32,address)` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `revokeOperator` | (`operator`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `revokeOperator(address)` | (`operator`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `revokeRole` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `revokeRole(bytes32,address)` | (`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `send` | (`recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `send(address,uint256,bytes)` | (`recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `symbol` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `symbol()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `totalSupply` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `totalSupply()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `totalSupplyAt` | (`_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `totalSupplyAt(uint128)` | (`_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `totalSupplySnapshots` | (`arg0`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `totalSupplySnapshots(uint256)` | (`arg0`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `transfer` | (`recipient`: *string*, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transfer(address,uint256)` | (`recipient`: *string*, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transferFrom` | (`holder`: *string*, `recipient`: *string*, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `transferFrom(address,address,uint256)` | (`holder`: *string*, `recipient`: *string*, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |

Overrides: Contract.populateTransaction

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:1749

___

### provider

• `Readonly` **provider**: *Provider*

Inherited from: Contract.provider

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:74

___

### resolvedAddress

• `Readonly` **resolvedAddress**: *Promise*<string\>

Inherited from: Contract.resolvedAddress

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:90

___

### signer

• `Readonly` **signer**: *Signer*

Inherited from: Contract.signer

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:73

## Methods

### DEFAULT\_ADMIN\_ROLE

▸ **DEFAULT_ADMIN_ROLE**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:662

___

### DEFAULT\_ADMIN\_ROLE()

▸ **DEFAULT_ADMIN_ROLE()**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:662

___

### MINTER\_ROLE

▸ **MINTER_ROLE**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:666

___

### MINTER\_ROLE()

▸ **MINTER_ROLE()**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:666

___

### \_checkRunningEvents

▸ **_checkRunningEvents**(`runningEvent`: *RunningEvent*): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `runningEvent` | *RunningEvent* |

**Returns:** *void*

Inherited from: Contract.\_checkRunningEvents

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:113

___

### \_deployed

▸ **_deployed**(`blockTag?`: BlockTag): *Promise*<Contract\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockTag?` | BlockTag |

**Returns:** *Promise*<Contract\>

Inherited from: Contract.\_deployed

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:106

___

### \_wrapEvent

▸ **_wrapEvent**(`runningEvent`: *RunningEvent*, `log`: Log, `listener`: Listener): Event

#### Parameters

| Name | Type |
| :------ | :------ |
| `runningEvent` | *RunningEvent* |
| `log` | Log |
| `listener` | Listener |

**Returns:** Event

Inherited from: Contract.\_wrapEvent

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:114

___

### accountSnapshots

▸ **accountSnapshots**(`arg0`: *string*, `arg1`: BigNumberish, `overrides?`: CallOverrides): *Promise*<[*BigNumber*, *BigNumber*] & { `fromBlock`: *BigNumber* ; `value`: *BigNumber*  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | *string* |
| `arg1` | BigNumberish |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<[*BigNumber*, *BigNumber*] & { `fromBlock`: *BigNumber* ; `value`: *BigNumber*  }\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:670

___

### accountSnapshots(address,uint256)

▸ **accountSnapshots(address,uint256)**(`arg0`: *string*, `arg1`: BigNumberish, `overrides?`: CallOverrides): *Promise*<[*BigNumber*, *BigNumber*] & { `fromBlock`: *BigNumber* ; `value`: *BigNumber*  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | *string* |
| `arg1` | BigNumberish |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<[*BigNumber*, *BigNumber*] & { `fromBlock`: *BigNumber* ; `value`: *BigNumber*  }\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:676

___

### allowance

▸ **allowance**(`holder`: *string*, `spender`: *string*, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `holder` | *string* |
| `spender` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:686

___

### allowance(address,address)

▸ **allowance(address,address)**(`holder`: *string*, `spender`: *string*, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `holder` | *string* |
| `spender` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:690

___

### approve

▸ **approve**(`spender`: *string*, `value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `spender` | *string* |
| `value` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:698

___

### approve(address,uint256)

▸ **approve(address,uint256)**(`spender`: *string*, `value`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `spender` | *string* |
| `value` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:702

___

### attach

▸ **attach**(`addressOrName`: *string*): [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |

**Returns:** [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.attach

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:290

___

### authorizeOperator

▸ **authorizeOperator**(`operator`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:710

___

### authorizeOperator(address)

▸ **authorizeOperator(address)**(`operator`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:713

___

### balanceOf

▸ **balanceOf**(`tokenHolder`: *string*, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `tokenHolder` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:720

___

### balanceOf(address)

▸ **balanceOf(address)**(`tokenHolder`: *string*, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `tokenHolder` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:720

___

### balanceOfAt

▸ **balanceOfAt**(`_owner`: *string*, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | *string* |
| `_blockNumber` | BigNumberish |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:727

___

### balanceOfAt(address,uint128)

▸ **balanceOfAt(address,uint128)**(`_owner`: *string*, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_owner` | *string* |
| `_blockNumber` | BigNumberish |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:731

___

### burn

▸ **burn**(`amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `amount` | BigNumberish |
| `data` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:739

___

### burn(uint256,bytes)

▸ **burn(uint256,bytes)**(`amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `amount` | BigNumberish |
| `data` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:743

___

### connect

▸ **connect**(`signerOrProvider`: *string* \| *Provider* \| *Signer*): [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | *string* \| *Provider* \| *Signer* |

**Returns:** [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.connect

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:289

___

### decimals

▸ **decimals**(`overrides?`: CallOverrides): *Promise*<number\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<number\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:751

___

### decimals()

▸ **decimals()**(`overrides?`: CallOverrides): *Promise*<number\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<number\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:751

___

### defaultOperators

▸ **defaultOperators**(`overrides?`: CallOverrides): *Promise*<string[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string[]\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:755

___

### defaultOperators()

▸ **defaultOperators()**(`overrides?`: CallOverrides): *Promise*<string[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string[]\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:755

___

### deployed

▸ **deployed**(): *Promise*<[*HoprToken*](contracts_hoprtoken.hoprtoken.md)\>

**Returns:** *Promise*<[*HoprToken*](contracts_hoprtoken.hoprtoken.md)\>

Overrides: Contract.deployed

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:291

___

### emit

▸ **emit**(`eventName`: *string* \| EventFilter, ...`args`: *any*[]): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* \| EventFilter |
| `...args` | *any*[] |

**Returns:** *boolean*

Inherited from: Contract.emit

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:119

___

### fallback

▸ **fallback**(`overrides?`: TransactionRequest): *Promise*<TransactionResponse\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | TransactionRequest |

**Returns:** *Promise*<TransactionResponse\>

Inherited from: Contract.fallback

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:107

___

### getRoleAdmin

▸ **getRoleAdmin**(`role`: BytesLike, `overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | BytesLike |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:759

___

### getRoleAdmin(bytes32)

▸ **getRoleAdmin(bytes32)**(`role`: BytesLike, `overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | BytesLike |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:759

___

### getRoleMember

▸ **getRoleMember**(`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | BytesLike |
| `index` | BigNumberish |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:766

___

### getRoleMember(bytes32,uint256)

▸ **getRoleMember(bytes32,uint256)**(`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | BytesLike |
| `index` | BigNumberish |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:770

___

### getRoleMemberCount

▸ **getRoleMemberCount**(`role`: BytesLike, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | BytesLike |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:778

___

### getRoleMemberCount(bytes32)

▸ **getRoleMemberCount(bytes32)**(`role`: BytesLike, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | BytesLike |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:781

___

### grantRole

▸ **grantRole**(`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | BytesLike |
| `account` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:788

___

### grantRole(bytes32,address)

▸ **grantRole(bytes32,address)**(`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | BytesLike |
| `account` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:792

___

### granularity

▸ **granularity**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:800

___

### granularity()

▸ **granularity()**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:800

___

### hasRole

▸ **hasRole**(`role`: BytesLike, `account`: *string*, `overrides?`: CallOverrides): *Promise*<boolean\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | BytesLike |
| `account` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<boolean\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:804

___

### hasRole(bytes32,address)

▸ **hasRole(bytes32,address)**(`role`: BytesLike, `account`: *string*, `overrides?`: CallOverrides): *Promise*<boolean\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | BytesLike |
| `account` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<boolean\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:808

___

### isOperatorFor

▸ **isOperatorFor**(`operator`: *string*, `tokenHolder`: *string*, `overrides?`: CallOverrides): *Promise*<boolean\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | *string* |
| `tokenHolder` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<boolean\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:816

___

### isOperatorFor(address,address)

▸ **isOperatorFor(address,address)**(`operator`: *string*, `tokenHolder`: *string*, `overrides?`: CallOverrides): *Promise*<boolean\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | *string* |
| `tokenHolder` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<boolean\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:820

___

### listenerCount

▸ **listenerCount**(`eventName?`: *string* \| EventFilter): *number*

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* \| EventFilter |

**Returns:** *number*

Inherited from: Contract.listenerCount

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:120

___

### listeners

▸ **listeners**<EventArgsArray, EventArgsObject\>(`eventFilter?`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>[]

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter?` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>[]

Overrides: Contract.listeners

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:293

▸ **listeners**(`eventName?`: *string*): Listener[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:316

___

### mint

▸ **mint**(`account`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | *string* |
| `amount` | BigNumberish |
| `userData` | BytesLike |
| `operatorData` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:828

___

### mint(address,uint256,bytes,bytes)

▸ **mint(address,uint256,bytes,bytes)**(`account`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | *string* |
| `amount` | BigNumberish |
| `userData` | BytesLike |
| `operatorData` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:834

___

### name

▸ **name**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:844

___

### name()

▸ **name()**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:844

___

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\> |

**Returns:** [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.off

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:296

▸ **off**(`eventName`: *string*, `listener`: Listener): [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.off

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:317

___

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\> |

**Returns:** [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.on

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:300

▸ **on**(`eventName`: *string*, `listener`: Listener): [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.on

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:318

___

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\> |

**Returns:** [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.once

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:304

▸ **once**(`eventName`: *string*, `listener`: Listener): [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.once

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:319

___

### operatorBurn

▸ **operatorBurn**(`account`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | *string* |
| `amount` | BigNumberish |
| `data` | BytesLike |
| `operatorData` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:848

___

### operatorBurn(address,uint256,bytes,bytes)

▸ **operatorBurn(address,uint256,bytes,bytes)**(`account`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | *string* |
| `amount` | BigNumberish |
| `data` | BytesLike |
| `operatorData` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:854

___

### operatorSend

▸ **operatorSend**(`sender`: *string*, `recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `sender` | *string* |
| `recipient` | *string* |
| `amount` | BigNumberish |
| `data` | BytesLike |
| `operatorData` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:864

___

### operatorSend(address,address,uint256,bytes,bytes)

▸ **operatorSend(address,address,uint256,bytes,bytes)**(`sender`: *string*, `recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `sender` | *string* |
| `recipient` | *string* |
| `amount` | BigNumberish |
| `data` | BytesLike |
| `operatorData` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:871

___

### queryFilter

▸ **queryFilter**<EventArgsArray, EventArgsObject\>(`event`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `fromBlockOrBlockhash?`: *string* \| *number*, `toBlock?`: *string* \| *number*): *Promise*<[*TypedEvent*](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>[]\>

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `fromBlockOrBlockhash?` | *string* \| *number* |
| `toBlock?` | *string* \| *number* |

**Returns:** *Promise*<[*TypedEvent*](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>[]\>

Overrides: Contract.queryFilter

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:323

___

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.removeAllListeners

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:312

▸ **removeAllListeners**(`eventName?`: *string*): [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.removeAllListeners

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:321

___

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\> |

**Returns:** [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.removeListener

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:308

▸ **removeListener**(`eventName`: *string*, `listener`: Listener): [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*HoprToken*](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.removeListener

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:320

___

### renounceRole

▸ **renounceRole**(`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | BytesLike |
| `account` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:882

___

### renounceRole(bytes32,address)

▸ **renounceRole(bytes32,address)**(`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | BytesLike |
| `account` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:886

___

### revokeOperator

▸ **revokeOperator**(`operator`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:894

___

### revokeOperator(address)

▸ **revokeOperator(address)**(`operator`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:897

___

### revokeRole

▸ **revokeRole**(`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | BytesLike |
| `account` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:904

___

### revokeRole(bytes32,address)

▸ **revokeRole(bytes32,address)**(`role`: BytesLike, `account`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `role` | BytesLike |
| `account` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:908

___

### send

▸ **send**(`recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `recipient` | *string* |
| `amount` | BigNumberish |
| `data` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:916

___

### send(address,uint256,bytes)

▸ **send(address,uint256,bytes)**(`recipient`: *string*, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `recipient` | *string* |
| `amount` | BigNumberish |
| `data` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:921

___

### symbol

▸ **symbol**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:930

___

### symbol()

▸ **symbol()**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:930

___

### totalSupply

▸ **totalSupply**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:934

___

### totalSupply()

▸ **totalSupply()**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:934

___

### totalSupplyAt

▸ **totalSupplyAt**(`_blockNumber`: BigNumberish, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_blockNumber` | BigNumberish |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:938

___

### totalSupplyAt(uint128)

▸ **totalSupplyAt(uint128)**(`_blockNumber`: BigNumberish, `overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_blockNumber` | BigNumberish |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:941

___

### totalSupplySnapshots

▸ **totalSupplySnapshots**(`arg0`: BigNumberish, `overrides?`: CallOverrides): *Promise*<[*BigNumber*, *BigNumber*] & { `fromBlock`: *BigNumber* ; `value`: *BigNumber*  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | BigNumberish |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<[*BigNumber*, *BigNumber*] & { `fromBlock`: *BigNumber* ; `value`: *BigNumber*  }\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:948

___

### totalSupplySnapshots(uint256)

▸ **totalSupplySnapshots(uint256)**(`arg0`: BigNumberish, `overrides?`: CallOverrides): *Promise*<[*BigNumber*, *BigNumber*] & { `fromBlock`: *BigNumber* ; `value`: *BigNumber*  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | BigNumberish |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<[*BigNumber*, *BigNumber*] & { `fromBlock`: *BigNumber* ; `value`: *BigNumber*  }\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:953

___

### transfer

▸ **transfer**(`recipient`: *string*, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `recipient` | *string* |
| `amount` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:962

___

### transfer(address,uint256)

▸ **transfer(address,uint256)**(`recipient`: *string*, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `recipient` | *string* |
| `amount` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:966

___

### transferFrom

▸ **transferFrom**(`holder`: *string*, `recipient`: *string*, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `holder` | *string* |
| `recipient` | *string* |
| `amount` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:974

___

### transferFrom(address,address,uint256)

▸ **transferFrom(address,address,uint256)**(`holder`: *string*, `recipient`: *string*, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `holder` | *string* |
| `recipient` | *string* |
| `amount` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:979

___

### getContractAddress

▸ `Static` **getContractAddress**(`transaction`: { `from`: *string* ; `nonce`: BigNumberish  }): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `transaction` | *object* |
| `transaction.from` | *string* |
| `transaction.nonce` | BigNumberish |

**Returns:** *string*

Inherited from: Contract.getContractAddress

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:100

___

### getInterface

▸ `Static` **getInterface**(`contractInterface`: ContractInterface): *Interface*

#### Parameters

| Name | Type |
| :------ | :------ |
| `contractInterface` | ContractInterface |

**Returns:** *Interface*

Inherited from: Contract.getInterface

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:104

___

### isIndexed

▸ `Static` **isIndexed**(`value`: *any*): value is Indexed

#### Parameters

| Name | Type |
| :------ | :------ |
| `value` | *any* |

**Returns:** value is Indexed

Inherited from: Contract.isIndexed

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:110
