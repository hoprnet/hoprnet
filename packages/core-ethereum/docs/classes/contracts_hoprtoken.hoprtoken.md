[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprToken](../modules/contracts_hoprtoken.md) / HoprToken

# Class: HoprToken

[contracts/HoprToken](../modules/contracts_hoprtoken.md).HoprToken

## Hierarchy

- _Contract_

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

- [DEFAULT_ADMIN_ROLE](contracts_hoprtoken.hoprtoken.md#default_admin_role)
- [DEFAULT_ADMIN_ROLE()](<contracts_hoprtoken.hoprtoken.md#default_admin_role()>)
- [MINTER_ROLE](contracts_hoprtoken.hoprtoken.md#minter_role)
- [MINTER_ROLE()](<contracts_hoprtoken.hoprtoken.md#minter_role()>)
- [\_checkRunningEvents](contracts_hoprtoken.hoprtoken.md#_checkrunningevents)
- [\_deployed](contracts_hoprtoken.hoprtoken.md#_deployed)
- [\_wrapEvent](contracts_hoprtoken.hoprtoken.md#_wrapevent)
- [accountSnapshots](contracts_hoprtoken.hoprtoken.md#accountsnapshots)
- [accountSnapshots(address,uint256)](<contracts_hoprtoken.hoprtoken.md#accountsnapshots(address,uint256)>)
- [allowance](contracts_hoprtoken.hoprtoken.md#allowance)
- [allowance(address,address)](<contracts_hoprtoken.hoprtoken.md#allowance(address,address)>)
- [approve](contracts_hoprtoken.hoprtoken.md#approve)
- [approve(address,uint256)](<contracts_hoprtoken.hoprtoken.md#approve(address,uint256)>)
- [attach](contracts_hoprtoken.hoprtoken.md#attach)
- [authorizeOperator](contracts_hoprtoken.hoprtoken.md#authorizeoperator)
- [authorizeOperator(address)](<contracts_hoprtoken.hoprtoken.md#authorizeoperator(address)>)
- [balanceOf](contracts_hoprtoken.hoprtoken.md#balanceof)
- [balanceOf(address)](<contracts_hoprtoken.hoprtoken.md#balanceof(address)>)
- [balanceOfAt](contracts_hoprtoken.hoprtoken.md#balanceofat)
- [balanceOfAt(address,uint128)](<contracts_hoprtoken.hoprtoken.md#balanceofat(address,uint128)>)
- [burn](contracts_hoprtoken.hoprtoken.md#burn)
- [burn(uint256,bytes)](<contracts_hoprtoken.hoprtoken.md#burn(uint256,bytes)>)
- [connect](contracts_hoprtoken.hoprtoken.md#connect)
- [decimals](contracts_hoprtoken.hoprtoken.md#decimals)
- [decimals()](<contracts_hoprtoken.hoprtoken.md#decimals()>)
- [defaultOperators](contracts_hoprtoken.hoprtoken.md#defaultoperators)
- [defaultOperators()](<contracts_hoprtoken.hoprtoken.md#defaultoperators()>)
- [deployed](contracts_hoprtoken.hoprtoken.md#deployed)
- [emit](contracts_hoprtoken.hoprtoken.md#emit)
- [fallback](contracts_hoprtoken.hoprtoken.md#fallback)
- [getRoleAdmin](contracts_hoprtoken.hoprtoken.md#getroleadmin)
- [getRoleAdmin(bytes32)](<contracts_hoprtoken.hoprtoken.md#getroleadmin(bytes32)>)
- [getRoleMember](contracts_hoprtoken.hoprtoken.md#getrolemember)
- [getRoleMember(bytes32,uint256)](<contracts_hoprtoken.hoprtoken.md#getrolemember(bytes32,uint256)>)
- [getRoleMemberCount](contracts_hoprtoken.hoprtoken.md#getrolemembercount)
- [getRoleMemberCount(bytes32)](<contracts_hoprtoken.hoprtoken.md#getrolemembercount(bytes32)>)
- [grantRole](contracts_hoprtoken.hoprtoken.md#grantrole)
- [grantRole(bytes32,address)](<contracts_hoprtoken.hoprtoken.md#grantrole(bytes32,address)>)
- [granularity](contracts_hoprtoken.hoprtoken.md#granularity)
- [granularity()](<contracts_hoprtoken.hoprtoken.md#granularity()>)
- [hasRole](contracts_hoprtoken.hoprtoken.md#hasrole)
- [hasRole(bytes32,address)](<contracts_hoprtoken.hoprtoken.md#hasrole(bytes32,address)>)
- [isOperatorFor](contracts_hoprtoken.hoprtoken.md#isoperatorfor)
- [isOperatorFor(address,address)](<contracts_hoprtoken.hoprtoken.md#isoperatorfor(address,address)>)
- [listenerCount](contracts_hoprtoken.hoprtoken.md#listenercount)
- [listeners](contracts_hoprtoken.hoprtoken.md#listeners)
- [mint](contracts_hoprtoken.hoprtoken.md#mint)
- [mint(address,uint256,bytes,bytes)](<contracts_hoprtoken.hoprtoken.md#mint(address,uint256,bytes,bytes)>)
- [name](contracts_hoprtoken.hoprtoken.md#name)
- [name()](<contracts_hoprtoken.hoprtoken.md#name()>)
- [off](contracts_hoprtoken.hoprtoken.md#off)
- [on](contracts_hoprtoken.hoprtoken.md#on)
- [once](contracts_hoprtoken.hoprtoken.md#once)
- [operatorBurn](contracts_hoprtoken.hoprtoken.md#operatorburn)
- [operatorBurn(address,uint256,bytes,bytes)](<contracts_hoprtoken.hoprtoken.md#operatorburn(address,uint256,bytes,bytes)>)
- [operatorSend](contracts_hoprtoken.hoprtoken.md#operatorsend)
- [operatorSend(address,address,uint256,bytes,bytes)](<contracts_hoprtoken.hoprtoken.md#operatorsend(address,address,uint256,bytes,bytes)>)
- [queryFilter](contracts_hoprtoken.hoprtoken.md#queryfilter)
- [removeAllListeners](contracts_hoprtoken.hoprtoken.md#removealllisteners)
- [removeListener](contracts_hoprtoken.hoprtoken.md#removelistener)
- [renounceRole](contracts_hoprtoken.hoprtoken.md#renouncerole)
- [renounceRole(bytes32,address)](<contracts_hoprtoken.hoprtoken.md#renouncerole(bytes32,address)>)
- [revokeOperator](contracts_hoprtoken.hoprtoken.md#revokeoperator)
- [revokeOperator(address)](<contracts_hoprtoken.hoprtoken.md#revokeoperator(address)>)
- [revokeRole](contracts_hoprtoken.hoprtoken.md#revokerole)
- [revokeRole(bytes32,address)](<contracts_hoprtoken.hoprtoken.md#revokerole(bytes32,address)>)
- [send](contracts_hoprtoken.hoprtoken.md#send)
- [send(address,uint256,bytes)](<contracts_hoprtoken.hoprtoken.md#send(address,uint256,bytes)>)
- [symbol](contracts_hoprtoken.hoprtoken.md#symbol)
- [symbol()](<contracts_hoprtoken.hoprtoken.md#symbol()>)
- [totalSupply](contracts_hoprtoken.hoprtoken.md#totalsupply)
- [totalSupply()](<contracts_hoprtoken.hoprtoken.md#totalsupply()>)
- [totalSupplyAt](contracts_hoprtoken.hoprtoken.md#totalsupplyat)
- [totalSupplyAt(uint128)](<contracts_hoprtoken.hoprtoken.md#totalsupplyat(uint128)>)
- [totalSupplySnapshots](contracts_hoprtoken.hoprtoken.md#totalsupplysnapshots)
- [totalSupplySnapshots(uint256)](<contracts_hoprtoken.hoprtoken.md#totalsupplysnapshots(uint256)>)
- [transfer](contracts_hoprtoken.hoprtoken.md#transfer)
- [transfer(address,uint256)](<contracts_hoprtoken.hoprtoken.md#transfer(address,uint256)>)
- [transferFrom](contracts_hoprtoken.hoprtoken.md#transferfrom)
- [transferFrom(address,address,uint256)](<contracts_hoprtoken.hoprtoken.md#transferfrom(address,address,uint256)>)
- [getContractAddress](contracts_hoprtoken.hoprtoken.md#getcontractaddress)
- [getInterface](contracts_hoprtoken.hoprtoken.md#getinterface)
- [isIndexed](contracts_hoprtoken.hoprtoken.md#isindexed)

## Constructors

### constructor

\+ **new HoprToken**(`addressOrName`: _string_, `contractInterface`: ContractInterface, `signerOrProvider?`: _Provider_ \| _Signer_): [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

#### Parameters

| Name                | Type                   |
| :------------------ | :--------------------- |
| `addressOrName`     | _string_               |
| `contractInterface` | ContractInterface      |
| `signerOrProvider?` | _Provider_ \| _Signer_ |

**Returns:** [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

Inherited from: Contract.constructor

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:98

## Properties

### \_deployedPromise

• **\_deployedPromise**: _Promise_<Contract\>

Inherited from: Contract.\_deployedPromise

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:92

---

### \_runningEvents

• **\_runningEvents**: _object_

#### Type declaration

Inherited from: Contract.\_runningEvents

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:93

---

### \_wrappedEmits

• **\_wrappedEmits**: _object_

#### Type declaration

Inherited from: Contract.\_wrappedEmits

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:96

---

### address

• `Readonly` **address**: _string_

Inherited from: Contract.address

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:71

---

### callStatic

• **callStatic**: _object_

#### Type declaration

| Name                                                | Type                                                                                                                                                                  |
| :-------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `DEFAULT_ADMIN_ROLE`                                | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                   |
| `DEFAULT_ADMIN_ROLE()`                              | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                   |
| `MINTER_ROLE`                                       | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                   |
| `MINTER_ROLE()`                                     | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                   |
| `accountSnapshots`                                  | (`arg0`: _string_, `arg1`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*, *BigNumber*] & { `fromBlock`: _BigNumber_ ; `value`: _BigNumber_ }\> |
| `accountSnapshots(address,uint256)`                 | (`arg0`: _string_, `arg1`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*, *BigNumber*] & { `fromBlock`: _BigNumber_ ; `value`: _BigNumber_ }\> |
| `allowance`                                         | (`holder`: _string_, `spender`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                       |
| `allowance(address,address)`                        | (`holder`: _string_, `spender`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                       |
| `approve`                                           | (`spender`: _string_, `value`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<boolean\>                                                                      |
| `approve(address,uint256)`                          | (`spender`: _string_, `value`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<boolean\>                                                                      |
| `authorizeOperator`                                 | (`operator`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                               |
| `authorizeOperator(address)`                        | (`operator`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                               |
| `balanceOf`                                         | (`tokenHolder`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                       |
| `balanceOf(address)`                                | (`tokenHolder`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                       |
| `balanceOfAt`                                       | (`_owner`: _string_, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                              |
| `balanceOfAt(address,uint128)`                      | (`_owner`: _string_, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                              |
| `burn`                                              | (`amount`: BigNumberish, `data`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>                                                                          |
| `burn(uint256,bytes)`                               | (`amount`: BigNumberish, `data`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>                                                                          |
| `decimals`                                          | (`overrides?`: CallOverrides) => _Promise_<number\>                                                                                                                   |
| `decimals()`                                        | (`overrides?`: CallOverrides) => _Promise_<number\>                                                                                                                   |
| `defaultOperators`                                  | (`overrides?`: CallOverrides) => _Promise_<string[]\>                                                                                                                 |
| `defaultOperators()`                                | (`overrides?`: CallOverrides) => _Promise_<string[]\>                                                                                                                 |
| `getRoleAdmin`                                      | (`role`: BytesLike, `overrides?`: CallOverrides) => _Promise_<string\>                                                                                                |
| `getRoleAdmin(bytes32)`                             | (`role`: BytesLike, `overrides?`: CallOverrides) => _Promise_<string\>                                                                                                |
| `getRoleMember`                                     | (`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<string\>                                                                         |
| `getRoleMember(bytes32,uint256)`                    | (`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<string\>                                                                         |
| `getRoleMemberCount`                                | (`role`: BytesLike, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                             |
| `getRoleMemberCount(bytes32)`                       | (`role`: BytesLike, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                             |
| `grantRole`                                         | (`role`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                             |
| `grantRole(bytes32,address)`                        | (`role`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                             |
| `granularity`                                       | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                |
| `granularity()`                                     | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                |
| `hasRole`                                           | (`role`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<boolean\>                                                                          |
| `hasRole(bytes32,address)`                          | (`role`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<boolean\>                                                                          |
| `isOperatorFor`                                     | (`operator`: _string_, `tokenHolder`: _string_, `overrides?`: CallOverrides) => _Promise_<boolean\>                                                                   |
| `isOperatorFor(address,address)`                    | (`operator`: _string_, `tokenHolder`: _string_, `overrides?`: CallOverrides) => _Promise_<boolean\>                                                                   |
| `mint`                                              | (`account`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>                      |
| `mint(address,uint256,bytes,bytes)`                 | (`account`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>                      |
| `name`                                              | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                   |
| `name()`                                            | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                   |
| `operatorBurn`                                      | (`account`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>                          |
| `operatorBurn(address,uint256,bytes,bytes)`         | (`account`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>                          |
| `operatorSend`                                      | (`sender`: _string_, `recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>    |
| `operatorSend(address,address,uint256,bytes,bytes)` | (`sender`: _string_, `recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>    |
| `renounceRole`                                      | (`role`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                             |
| `renounceRole(bytes32,address)`                     | (`role`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                             |
| `revokeOperator`                                    | (`operator`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                               |
| `revokeOperator(address)`                           | (`operator`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                               |
| `revokeRole`                                        | (`role`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                             |
| `revokeRole(bytes32,address)`                       | (`role`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                             |
| `send`                                              | (`recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>                                                   |
| `send(address,uint256,bytes)`                       | (`recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>                                                   |
| `symbol`                                            | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                   |
| `symbol()`                                          | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                   |
| `totalSupply`                                       | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                |
| `totalSupply()`                                     | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                |
| `totalSupplyAt`                                     | (`_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                  |
| `totalSupplyAt(uint128)`                            | (`_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                  |
| `totalSupplySnapshots`                              | (`arg0`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*, *BigNumber*] & { `fromBlock`: _BigNumber_ ; `value`: _BigNumber_ }\>                   |
| `totalSupplySnapshots(uint256)`                     | (`arg0`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*, *BigNumber*] & { `fromBlock`: _BigNumber_ ; `value`: _BigNumber_ }\>                   |
| `transfer`                                          | (`recipient`: _string_, `amount`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<boolean\>                                                                   |
| `transfer(address,uint256)`                         | (`recipient`: _string_, `amount`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<boolean\>                                                                   |
| `transferFrom`                                      | (`holder`: _string_, `recipient`: _string_, `amount`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<boolean\>                                               |
| `transferFrom(address,address,uint256)`             | (`holder`: _string_, `recipient`: _string_, `amount`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<boolean\>                                               |

Overrides: Contract.callStatic

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:988

---

### deployTransaction

• `Readonly` **deployTransaction**: TransactionResponse

Inherited from: Contract.deployTransaction

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:91

---

### estimateGas

• **estimateGas**: _object_

#### Type declaration

| Name                                                | Type                                                                                                                                                                                                              |
| :-------------------------------------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `DEFAULT_ADMIN_ROLE`                                | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                            |
| `DEFAULT_ADMIN_ROLE()`                              | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                            |
| `MINTER_ROLE`                                       | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                            |
| `MINTER_ROLE()`                                     | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                            |
| `accountSnapshots`                                  | (`arg0`: _string_, `arg1`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                    |
| `accountSnapshots(address,uint256)`                 | (`arg0`: _string_, `arg1`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                    |
| `allowance`                                         | (`holder`: _string_, `spender`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                   |
| `allowance(address,address)`                        | (`holder`: _string_, `spender`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                   |
| `approve`                                           | (`spender`: _string_, `value`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                      |
| `approve(address,uint256)`                          | (`spender`: _string_, `value`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                      |
| `authorizeOperator`                                 | (`operator`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                            |
| `authorizeOperator(address)`                        | (`operator`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                            |
| `balanceOf`                                         | (`tokenHolder`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                   |
| `balanceOf(address)`                                | (`tokenHolder`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                   |
| `balanceOfAt`                                       | (`_owner`: _string_, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                          |
| `balanceOfAt(address,uint128)`                      | (`_owner`: _string_, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                          |
| `burn`                                              | (`amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                       |
| `burn(uint256,bytes)`                               | (`amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                       |
| `decimals`                                          | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                            |
| `decimals()`                                        | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                            |
| `defaultOperators`                                  | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                            |
| `defaultOperators()`                                | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                            |
| `getRoleAdmin`                                      | (`role`: BytesLike, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                         |
| `getRoleAdmin(bytes32)`                             | (`role`: BytesLike, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                         |
| `getRoleMember`                                     | (`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                  |
| `getRoleMember(bytes32,uint256)`                    | (`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                  |
| `getRoleMemberCount`                                | (`role`: BytesLike, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                         |
| `getRoleMemberCount(bytes32)`                       | (`role`: BytesLike, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                         |
| `grantRole`                                         | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                          |
| `grantRole(bytes32,address)`                        | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                          |
| `granularity`                                       | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                            |
| `granularity()`                                     | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                            |
| `hasRole`                                           | (`role`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                    |
| `hasRole(bytes32,address)`                          | (`role`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                    |
| `isOperatorFor`                                     | (`operator`: _string_, `tokenHolder`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                             |
| `isOperatorFor(address,address)`                    | (`operator`: _string_, `tokenHolder`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                             |
| `mint`                                              | (`account`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                   |
| `mint(address,uint256,bytes,bytes)`                 | (`account`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                   |
| `name`                                              | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                            |
| `name()`                                            | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                            |
| `operatorBurn`                                      | (`account`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                       |
| `operatorBurn(address,uint256,bytes,bytes)`         | (`account`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                       |
| `operatorSend`                                      | (`sender`: _string_, `recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\> |
| `operatorSend(address,address,uint256,bytes,bytes)` | (`sender`: _string_, `recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\> |
| `renounceRole`                                      | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                          |
| `renounceRole(bytes32,address)`                     | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                          |
| `revokeOperator`                                    | (`operator`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                            |
| `revokeOperator(address)`                           | (`operator`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                            |
| `revokeRole`                                        | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                          |
| `revokeRole(bytes32,address)`                       | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                          |
| `send`                                              | (`recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                |
| `send(address,uint256,bytes)`                       | (`recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                |
| `symbol`                                            | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                            |
| `symbol()`                                          | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                            |
| `totalSupply`                                       | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                            |
| `totalSupply()`                                     | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                            |
| `totalSupplyAt`                                     | (`_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                              |
| `totalSupplyAt(uint128)`                            | (`_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                              |
| `totalSupplySnapshots`                              | (`arg0`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                      |
| `totalSupplySnapshots(uint256)`                     | (`arg0`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                      |
| `transfer`                                          | (`recipient`: _string_, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                   |
| `transfer(address,uint256)`                         | (`recipient`: _string_, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                   |
| `transferFrom`                                      | (`holder`: _string_, `recipient`: _string_, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                               |
| `transferFrom(address,address,uint256)`             | (`holder`: _string_, `recipient`: _string_, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                               |

Overrides: Contract.estimateGas

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:1423

---

### filters

• **filters**: _object_

#### Type declaration

| Name                 | Type                                                                                                                                                                                                                                                                                                                                                                                                  |
| :------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Approval`           | (`owner`: _string_, `spender`: _string_, `value`: `null`) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*, *BigNumber*], { `owner`: _string_ ; `spender`: _string_ ; `value`: _BigNumber_ }\>                                                                                                                                                        |
| `AuthorizedOperator` | (`operator`: _string_, `tokenHolder`: _string_) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*], { `operator`: _string_ ; `tokenHolder`: _string_ }\>                                                                                                                                                                                               |
| `Burned`             | (`operator`: _string_, `from`: _string_, `amount`: `null`, `data`: `null`, `operatorData`: `null`) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*, *BigNumber*, *string*, *string*], { `amount`: _BigNumber_ ; `data`: _string_ ; `from`: _string_ ; `operator`: _string_ ; `operatorData`: _string_ }\>                                            |
| `Minted`             | (`operator`: _string_, `to`: _string_, `amount`: `null`, `data`: `null`, `operatorData`: `null`) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*, *BigNumber*, *string*, *string*], { `amount`: _BigNumber_ ; `data`: _string_ ; `operator`: _string_ ; `operatorData`: _string_ ; `to`: _string_ }\>                                                |
| `RevokedOperator`    | (`operator`: _string_, `tokenHolder`: _string_) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*], { `operator`: _string_ ; `tokenHolder`: _string_ }\>                                                                                                                                                                                               |
| `RoleGranted`        | (`role`: BytesLike, `account`: _string_, `sender`: _string_) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*, *string*], { `account`: _string_ ; `role`: _string_ ; `sender`: _string_ }\>                                                                                                                                                           |
| `RoleRevoked`        | (`role`: BytesLike, `account`: _string_, `sender`: _string_) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*, *string*], { `account`: _string_ ; `role`: _string_ ; `sender`: _string_ }\>                                                                                                                                                           |
| `Sent`               | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: `null`, `data`: `null`, `operatorData`: `null`) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*, *string*, *BigNumber*, *string*, *string*], { `amount`: _BigNumber_ ; `data`: _string_ ; `from`: _string_ ; `operator`: _string_ ; `operatorData`: _string_ ; `to`: _string_ }\> |
| `Transfer`           | (`from`: _string_, `to`: _string_, `value`: `null`) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*, *BigNumber*], { `from`: _string_ ; `to`: _string_ ; `value`: _BigNumber_ }\>                                                                                                                                                                    |

Overrides: Contract.filters

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:1316

---

### functions

• **functions**: _object_

#### Type declaration

| Name                                                | Type                                                                                                                                                                                                                        |
| :-------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `DEFAULT_ADMIN_ROLE`                                | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                     |
| `DEFAULT_ADMIN_ROLE()`                              | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                     |
| `MINTER_ROLE`                                       | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                     |
| `MINTER_ROLE()`                                     | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                     |
| `accountSnapshots`                                  | (`arg0`: _string_, `arg1`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*, *BigNumber*] & { `fromBlock`: _BigNumber_ ; `value`: _BigNumber_ }\>                                                       |
| `accountSnapshots(address,uint256)`                 | (`arg0`: _string_, `arg1`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*, *BigNumber*] & { `fromBlock`: _BigNumber_ ; `value`: _BigNumber_ }\>                                                       |
| `allowance`                                         | (`holder`: _string_, `spender`: _string_, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                         |
| `allowance(address,address)`                        | (`holder`: _string_, `spender`: _string_, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                         |
| `approve`                                           | (`spender`: _string_, `value`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                      |
| `approve(address,uint256)`                          | (`spender`: _string_, `value`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                      |
| `authorizeOperator`                                 | (`operator`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                            |
| `authorizeOperator(address)`                        | (`operator`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                            |
| `balanceOf`                                         | (`tokenHolder`: _string_, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                         |
| `balanceOf(address)`                                | (`tokenHolder`: _string_, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                         |
| `balanceOfAt`                                       | (`_owner`: _string_, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                |
| `balanceOfAt(address,uint128)`                      | (`_owner`: _string_, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                |
| `burn`                                              | (`amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                       |
| `burn(uint256,bytes)`                               | (`amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                       |
| `decimals`                                          | (`overrides?`: CallOverrides) => _Promise_<[*number*]\>                                                                                                                                                                     |
| `decimals()`                                        | (`overrides?`: CallOverrides) => _Promise_<[*number*]\>                                                                                                                                                                     |
| `defaultOperators`                                  | (`overrides?`: CallOverrides) => _Promise_<[_string_[]]\>                                                                                                                                                                   |
| `defaultOperators()`                                | (`overrides?`: CallOverrides) => _Promise_<[_string_[]]\>                                                                                                                                                                   |
| `getRoleAdmin`                                      | (`role`: BytesLike, `overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                  |
| `getRoleAdmin(bytes32)`                             | (`role`: BytesLike, `overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                  |
| `getRoleMember`                                     | (`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                           |
| `getRoleMember(bytes32,uint256)`                    | (`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                           |
| `getRoleMemberCount`                                | (`role`: BytesLike, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                               |
| `getRoleMemberCount(bytes32)`                       | (`role`: BytesLike, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                               |
| `grantRole`                                         | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                          |
| `grantRole(bytes32,address)`                        | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                          |
| `granularity`                                       | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                  |
| `granularity()`                                     | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                  |
| `hasRole`                                           | (`role`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<[*boolean*]\>                                                                                                                            |
| `hasRole(bytes32,address)`                          | (`role`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<[*boolean*]\>                                                                                                                            |
| `isOperatorFor`                                     | (`operator`: _string_, `tokenHolder`: _string_, `overrides?`: CallOverrides) => _Promise_<[*boolean*]\>                                                                                                                     |
| `isOperatorFor(address,address)`                    | (`operator`: _string_, `tokenHolder`: _string_, `overrides?`: CallOverrides) => _Promise_<[*boolean*]\>                                                                                                                     |
| `mint`                                              | (`account`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                   |
| `mint(address,uint256,bytes,bytes)`                 | (`account`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                   |
| `name`                                              | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                     |
| `name()`                                            | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                     |
| `operatorBurn`                                      | (`account`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                       |
| `operatorBurn(address,uint256,bytes,bytes)`         | (`account`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                       |
| `operatorSend`                                      | (`sender`: _string_, `recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\> |
| `operatorSend(address,address,uint256,bytes,bytes)` | (`sender`: _string_, `recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\> |
| `renounceRole`                                      | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                          |
| `renounceRole(bytes32,address)`                     | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                          |
| `revokeOperator`                                    | (`operator`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                            |
| `revokeOperator(address)`                           | (`operator`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                            |
| `revokeRole`                                        | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                          |
| `revokeRole(bytes32,address)`                       | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                          |
| `send`                                              | (`recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                |
| `send(address,uint256,bytes)`                       | (`recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                |
| `symbol`                                            | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                     |
| `symbol()`                                          | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                     |
| `totalSupply`                                       | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                  |
| `totalSupply()`                                     | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                  |
| `totalSupplyAt`                                     | (`_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                    |
| `totalSupplyAt(uint128)`                            | (`_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                    |
| `totalSupplySnapshots`                              | (`arg0`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*, *BigNumber*] & { `fromBlock`: _BigNumber_ ; `value`: _BigNumber_ }\>                                                                         |
| `totalSupplySnapshots(uint256)`                     | (`arg0`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*, *BigNumber*] & { `fromBlock`: _BigNumber_ ; `value`: _BigNumber_ }\>                                                                         |
| `transfer`                                          | (`recipient`: _string_, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                   |
| `transfer(address,uint256)`                         | (`recipient`: _string_, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                   |
| `transferFrom`                                      | (`holder`: _string_, `recipient`: _string_, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                               |
| `transferFrom(address,address,uint256)`             | (`holder`: _string_, `recipient`: _string_, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                               |

Overrides: Contract.functions

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:331

---

### interface

• **interface**: [_HoprTokenInterface_](../interfaces/contracts_hoprtoken.hoprtokeninterface.md)

Overrides: Contract.interface

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:329

---

### populateTransaction

• **populateTransaction**: _object_

#### Type declaration

| Name                                                | Type                                                                                                                                                                                                                         |
| :-------------------------------------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `DEFAULT_ADMIN_ROLE`                                | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                            |
| `DEFAULT_ADMIN_ROLE()`                              | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                            |
| `MINTER_ROLE`                                       | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                            |
| `MINTER_ROLE()`                                     | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                            |
| `accountSnapshots`                                  | (`arg0`: _string_, `arg1`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                    |
| `accountSnapshots(address,uint256)`                 | (`arg0`: _string_, `arg1`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                    |
| `allowance`                                         | (`holder`: _string_, `spender`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                   |
| `allowance(address,address)`                        | (`holder`: _string_, `spender`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                   |
| `approve`                                           | (`spender`: _string_, `value`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                      |
| `approve(address,uint256)`                          | (`spender`: _string_, `value`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                      |
| `authorizeOperator`                                 | (`operator`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                            |
| `authorizeOperator(address)`                        | (`operator`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                            |
| `balanceOf`                                         | (`tokenHolder`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                   |
| `balanceOf(address)`                                | (`tokenHolder`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                   |
| `balanceOfAt`                                       | (`_owner`: _string_, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                          |
| `balanceOfAt(address,uint128)`                      | (`_owner`: _string_, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                          |
| `burn`                                              | (`amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                       |
| `burn(uint256,bytes)`                               | (`amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                       |
| `decimals`                                          | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                            |
| `decimals()`                                        | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                            |
| `defaultOperators`                                  | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                            |
| `defaultOperators()`                                | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                            |
| `getRoleAdmin`                                      | (`role`: BytesLike, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                         |
| `getRoleAdmin(bytes32)`                             | (`role`: BytesLike, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                         |
| `getRoleMember`                                     | (`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                  |
| `getRoleMember(bytes32,uint256)`                    | (`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                  |
| `getRoleMemberCount`                                | (`role`: BytesLike, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                         |
| `getRoleMemberCount(bytes32)`                       | (`role`: BytesLike, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                         |
| `grantRole`                                         | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                          |
| `grantRole(bytes32,address)`                        | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                          |
| `granularity`                                       | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                            |
| `granularity()`                                     | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                            |
| `hasRole`                                           | (`role`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                    |
| `hasRole(bytes32,address)`                          | (`role`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                    |
| `isOperatorFor`                                     | (`operator`: _string_, `tokenHolder`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                             |
| `isOperatorFor(address,address)`                    | (`operator`: _string_, `tokenHolder`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                             |
| `mint`                                              | (`account`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                   |
| `mint(address,uint256,bytes,bytes)`                 | (`account`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                   |
| `name`                                              | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                            |
| `name()`                                            | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                            |
| `operatorBurn`                                      | (`account`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                       |
| `operatorBurn(address,uint256,bytes,bytes)`         | (`account`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                       |
| `operatorSend`                                      | (`sender`: _string_, `recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\> |
| `operatorSend(address,address,uint256,bytes,bytes)` | (`sender`: _string_, `recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\> |
| `renounceRole`                                      | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                          |
| `renounceRole(bytes32,address)`                     | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                          |
| `revokeOperator`                                    | (`operator`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                            |
| `revokeOperator(address)`                           | (`operator`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                            |
| `revokeRole`                                        | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                          |
| `revokeRole(bytes32,address)`                       | (`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                          |
| `send`                                              | (`recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                |
| `send(address,uint256,bytes)`                       | (`recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                |
| `symbol`                                            | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                            |
| `symbol()`                                          | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                            |
| `totalSupply`                                       | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                            |
| `totalSupply()`                                     | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                            |
| `totalSupplyAt`                                     | (`_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                              |
| `totalSupplyAt(uint128)`                            | (`_blockNumber`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                              |
| `totalSupplySnapshots`                              | (`arg0`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                      |
| `totalSupplySnapshots(uint256)`                     | (`arg0`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                      |
| `transfer`                                          | (`recipient`: _string_, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                   |
| `transfer(address,uint256)`                         | (`recipient`: _string_, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                   |
| `transferFrom`                                      | (`holder`: _string_, `recipient`: _string_, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                               |
| `transferFrom(address,address,uint256)`             | (`holder`: _string_, `recipient`: _string_, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                               |

Overrides: Contract.populateTransaction

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:1749

---

### provider

• `Readonly` **provider**: _Provider_

Inherited from: Contract.provider

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:74

---

### resolvedAddress

• `Readonly` **resolvedAddress**: _Promise_<string\>

Inherited from: Contract.resolvedAddress

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:90

---

### signer

• `Readonly` **signer**: _Signer_

Inherited from: Contract.signer

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:73

## Methods

### DEFAULT_ADMIN_ROLE

▸ **DEFAULT_ADMIN_ROLE**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:662

---

### DEFAULT_ADMIN_ROLE()

▸ **DEFAULT_ADMIN_ROLE()**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:662

---

### MINTER_ROLE

▸ **MINTER_ROLE**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:666

---

### MINTER_ROLE()

▸ **MINTER_ROLE()**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:666

---

### \_checkRunningEvents

▸ **\_checkRunningEvents**(`runningEvent`: _RunningEvent_): _void_

#### Parameters

| Name           | Type           |
| :------------- | :------------- |
| `runningEvent` | _RunningEvent_ |

**Returns:** _void_

Inherited from: Contract.\_checkRunningEvents

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:113

---

### \_deployed

▸ **\_deployed**(`blockTag?`: BlockTag): _Promise_<Contract\>

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `blockTag?` | BlockTag |

**Returns:** _Promise_<Contract\>

Inherited from: Contract.\_deployed

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:106

---

### \_wrapEvent

▸ **\_wrapEvent**(`runningEvent`: _RunningEvent_, `log`: Log, `listener`: Listener): Event

#### Parameters

| Name           | Type           |
| :------------- | :------------- |
| `runningEvent` | _RunningEvent_ |
| `log`          | Log            |
| `listener`     | Listener       |

**Returns:** Event

Inherited from: Contract.\_wrapEvent

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:114

---

### accountSnapshots

▸ **accountSnapshots**(`arg0`: _string_, `arg1`: BigNumberish, `overrides?`: CallOverrides): _Promise_<[*BigNumber*, *BigNumber*] & { `fromBlock`: _BigNumber_ ; `value`: _BigNumber_ }\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `arg0`       | _string_      |
| `arg1`       | BigNumberish  |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<[*BigNumber*, *BigNumber*] & { `fromBlock`: _BigNumber_ ; `value`: _BigNumber_ }\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:670

---

### accountSnapshots(address,uint256)

▸ **accountSnapshots(address,uint256)**(`arg0`: _string_, `arg1`: BigNumberish, `overrides?`: CallOverrides): _Promise_<[*BigNumber*, *BigNumber*] & { `fromBlock`: _BigNumber_ ; `value`: _BigNumber_ }\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `arg0`       | _string_      |
| `arg1`       | BigNumberish  |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<[*BigNumber*, *BigNumber*] & { `fromBlock`: _BigNumber_ ; `value`: _BigNumber_ }\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:676

---

### allowance

▸ **allowance**(`holder`: _string_, `spender`: _string_, `overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `holder`     | _string_      |
| `spender`    | _string_      |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:686

---

### allowance(address,address)

▸ **allowance(address,address)**(`holder`: _string_, `spender`: _string_, `overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `holder`     | _string_      |
| `spender`    | _string_      |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:690

---

### approve

▸ **approve**(`spender`: _string_, `value`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `spender`    | _string_                                                |
| `value`      | BigNumberish                                            |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:698

---

### approve(address,uint256)

▸ **approve(address,uint256)**(`spender`: _string_, `value`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `spender`    | _string_                                                |
| `value`      | BigNumberish                                            |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:702

---

### attach

▸ **attach**(`addressOrName`: _string_): [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

#### Parameters

| Name            | Type     |
| :-------------- | :------- |
| `addressOrName` | _string_ |

**Returns:** [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.attach

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:290

---

### authorizeOperator

▸ **authorizeOperator**(`operator`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `operator`   | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:710

---

### authorizeOperator(address)

▸ **authorizeOperator(address)**(`operator`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `operator`   | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:713

---

### balanceOf

▸ **balanceOf**(`tokenHolder`: _string_, `overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name          | Type          |
| :------------ | :------------ |
| `tokenHolder` | _string_      |
| `overrides?`  | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:720

---

### balanceOf(address)

▸ **balanceOf(address)**(`tokenHolder`: _string_, `overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name          | Type          |
| :------------ | :------------ |
| `tokenHolder` | _string_      |
| `overrides?`  | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:720

---

### balanceOfAt

▸ **balanceOfAt**(`_owner`: _string_, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name           | Type          |
| :------------- | :------------ |
| `_owner`       | _string_      |
| `_blockNumber` | BigNumberish  |
| `overrides?`   | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:727

---

### balanceOfAt(address,uint128)

▸ **balanceOfAt(address,uint128)**(`_owner`: _string_, `_blockNumber`: BigNumberish, `overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name           | Type          |
| :------------- | :------------ |
| `_owner`       | _string_      |
| `_blockNumber` | BigNumberish  |
| `overrides?`   | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:731

---

### burn

▸ **burn**(`amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `amount`     | BigNumberish                                            |
| `data`       | BytesLike                                               |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:739

---

### burn(uint256,bytes)

▸ **burn(uint256,bytes)**(`amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `amount`     | BigNumberish                                            |
| `data`       | BytesLike                                               |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:743

---

### connect

▸ **connect**(`signerOrProvider`: _string_ \| _Provider_ \| _Signer_): [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

#### Parameters

| Name               | Type                               |
| :----------------- | :--------------------------------- |
| `signerOrProvider` | _string_ \| _Provider_ \| _Signer_ |

**Returns:** [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.connect

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:289

---

### decimals

▸ **decimals**(`overrides?`: CallOverrides): _Promise_<number\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<number\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:751

---

### decimals()

▸ **decimals()**(`overrides?`: CallOverrides): _Promise_<number\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<number\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:751

---

### defaultOperators

▸ **defaultOperators**(`overrides?`: CallOverrides): _Promise_<string[]\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string[]\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:755

---

### defaultOperators()

▸ **defaultOperators()**(`overrides?`: CallOverrides): _Promise_<string[]\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string[]\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:755

---

### deployed

▸ **deployed**(): _Promise_<[_HoprToken_](contracts_hoprtoken.hoprtoken.md)\>

**Returns:** _Promise_<[_HoprToken_](contracts_hoprtoken.hoprtoken.md)\>

Overrides: Contract.deployed

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:291

---

### emit

▸ **emit**(`eventName`: _string_ \| EventFilter, ...`args`: _any_[]): _boolean_

#### Parameters

| Name        | Type                    |
| :---------- | :---------------------- |
| `eventName` | _string_ \| EventFilter |
| `...args`   | _any_[]                 |

**Returns:** _boolean_

Inherited from: Contract.emit

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:119

---

### fallback

▸ **fallback**(`overrides?`: TransactionRequest): _Promise_<TransactionResponse\>

#### Parameters

| Name         | Type               |
| :----------- | :----------------- |
| `overrides?` | TransactionRequest |

**Returns:** _Promise_<TransactionResponse\>

Inherited from: Contract.fallback

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:107

---

### getRoleAdmin

▸ **getRoleAdmin**(`role`: BytesLike, `overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `role`       | BytesLike     |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:759

---

### getRoleAdmin(bytes32)

▸ **getRoleAdmin(bytes32)**(`role`: BytesLike, `overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `role`       | BytesLike     |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:759

---

### getRoleMember

▸ **getRoleMember**(`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `role`       | BytesLike     |
| `index`      | BigNumberish  |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:766

---

### getRoleMember(bytes32,uint256)

▸ **getRoleMember(bytes32,uint256)**(`role`: BytesLike, `index`: BigNumberish, `overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `role`       | BytesLike     |
| `index`      | BigNumberish  |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:770

---

### getRoleMemberCount

▸ **getRoleMemberCount**(`role`: BytesLike, `overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `role`       | BytesLike     |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:778

---

### getRoleMemberCount(bytes32)

▸ **getRoleMemberCount(bytes32)**(`role`: BytesLike, `overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `role`       | BytesLike     |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:781

---

### grantRole

▸ **grantRole**(`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `role`       | BytesLike                                               |
| `account`    | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:788

---

### grantRole(bytes32,address)

▸ **grantRole(bytes32,address)**(`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `role`       | BytesLike                                               |
| `account`    | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:792

---

### granularity

▸ **granularity**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:800

---

### granularity()

▸ **granularity()**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:800

---

### hasRole

▸ **hasRole**(`role`: BytesLike, `account`: _string_, `overrides?`: CallOverrides): _Promise_<boolean\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `role`       | BytesLike     |
| `account`    | _string_      |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<boolean\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:804

---

### hasRole(bytes32,address)

▸ **hasRole(bytes32,address)**(`role`: BytesLike, `account`: _string_, `overrides?`: CallOverrides): _Promise_<boolean\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `role`       | BytesLike     |
| `account`    | _string_      |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<boolean\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:808

---

### isOperatorFor

▸ **isOperatorFor**(`operator`: _string_, `tokenHolder`: _string_, `overrides?`: CallOverrides): _Promise_<boolean\>

#### Parameters

| Name          | Type          |
| :------------ | :------------ |
| `operator`    | _string_      |
| `tokenHolder` | _string_      |
| `overrides?`  | CallOverrides |

**Returns:** _Promise_<boolean\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:816

---

### isOperatorFor(address,address)

▸ **isOperatorFor(address,address)**(`operator`: _string_, `tokenHolder`: _string_, `overrides?`: CallOverrides): _Promise_<boolean\>

#### Parameters

| Name          | Type          |
| :------------ | :------------ |
| `operator`    | _string_      |
| `tokenHolder` | _string_      |
| `overrides?`  | CallOverrides |

**Returns:** _Promise_<boolean\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:820

---

### listenerCount

▸ **listenerCount**(`eventName?`: _string_ \| EventFilter): _number_

#### Parameters

| Name         | Type                    |
| :----------- | :---------------------- |
| `eventName?` | _string_ \| EventFilter |

**Returns:** _number_

Inherited from: Contract.listenerCount

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:120

---

### listeners

▸ **listeners**<EventArgsArray, EventArgsObject\>(`eventFilter?`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>[]

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name           | Type                                                                                                        |
| :------------- | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter?` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>[]

Overrides: Contract.listeners

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:293

▸ **listeners**(`eventName?`: _string_): Listener[]

#### Parameters

| Name         | Type     |
| :----------- | :------- |
| `eventName?` | _string_ |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:316

---

### mint

▸ **mint**(`account`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `account`      | _string_                                                |
| `amount`       | BigNumberish                                            |
| `userData`     | BytesLike                                               |
| `operatorData` | BytesLike                                               |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:828

---

### mint(address,uint256,bytes,bytes)

▸ **mint(address,uint256,bytes,bytes)**(`account`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `account`      | _string_                                                |
| `amount`       | BigNumberish                                            |
| `userData`     | BytesLike                                               |
| `operatorData` | BytesLike                                               |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:834

---

### name

▸ **name**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:844

---

### name()

▸ **name()**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:844

---

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name          | Type                                                                                                        |
| :------------ | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener`    | [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>          |

**Returns:** [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.off

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:296

▸ **off**(`eventName`: _string_, `listener`: Listener): [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.off

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:317

---

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name          | Type                                                                                                        |
| :------------ | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener`    | [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>          |

**Returns:** [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.on

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:300

▸ **on**(`eventName`: _string_, `listener`: Listener): [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.on

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:318

---

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name          | Type                                                                                                        |
| :------------ | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener`    | [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>          |

**Returns:** [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.once

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:304

▸ **once**(`eventName`: _string_, `listener`: Listener): [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.once

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:319

---

### operatorBurn

▸ **operatorBurn**(`account`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `account`      | _string_                                                |
| `amount`       | BigNumberish                                            |
| `data`         | BytesLike                                               |
| `operatorData` | BytesLike                                               |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:848

---

### operatorBurn(address,uint256,bytes,bytes)

▸ **operatorBurn(address,uint256,bytes,bytes)**(`account`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `account`      | _string_                                                |
| `amount`       | BigNumberish                                            |
| `data`         | BytesLike                                               |
| `operatorData` | BytesLike                                               |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:854

---

### operatorSend

▸ **operatorSend**(`sender`: _string_, `recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `sender`       | _string_                                                |
| `recipient`    | _string_                                                |
| `amount`       | BigNumberish                                            |
| `data`         | BytesLike                                               |
| `operatorData` | BytesLike                                               |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:864

---

### operatorSend(address,address,uint256,bytes,bytes)

▸ **operatorSend(address,address,uint256,bytes,bytes)**(`sender`: _string_, `recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `sender`       | _string_                                                |
| `recipient`    | _string_                                                |
| `amount`       | BigNumberish                                            |
| `data`         | BytesLike                                               |
| `operatorData` | BytesLike                                               |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:871

---

### queryFilter

▸ **queryFilter**<EventArgsArray, EventArgsObject\>(`event`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `fromBlockOrBlockhash?`: _string_ \| _number_, `toBlock?`: _string_ \| _number_): _Promise_<[_TypedEvent_](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>[]\>

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name                    | Type                                                                                                        |
| :---------------------- | :---------------------------------------------------------------------------------------------------------- |
| `event`                 | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `fromBlockOrBlockhash?` | _string_ \| _number_                                                                                        |
| `toBlock?`              | _string_ \| _number_                                                                                        |

**Returns:** _Promise_<[_TypedEvent_](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>[]\>

Overrides: Contract.queryFilter

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:323

---

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name          | Type                                                                                                        |
| :------------ | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.removeAllListeners

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:312

▸ **removeAllListeners**(`eventName?`: _string_): [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

#### Parameters

| Name         | Type     |
| :----------- | :------- |
| `eventName?` | _string_ |

**Returns:** [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.removeAllListeners

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:321

---

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name          | Type                                                                                                        |
| :------------ | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener`    | [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>          |

**Returns:** [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.removeListener

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:308

▸ **removeListener**(`eventName`: _string_, `listener`: Listener): [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprToken_](contracts_hoprtoken.hoprtoken.md)

Overrides: Contract.removeListener

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:320

---

### renounceRole

▸ **renounceRole**(`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `role`       | BytesLike                                               |
| `account`    | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:882

---

### renounceRole(bytes32,address)

▸ **renounceRole(bytes32,address)**(`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `role`       | BytesLike                                               |
| `account`    | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:886

---

### revokeOperator

▸ **revokeOperator**(`operator`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `operator`   | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:894

---

### revokeOperator(address)

▸ **revokeOperator(address)**(`operator`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `operator`   | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:897

---

### revokeRole

▸ **revokeRole**(`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `role`       | BytesLike                                               |
| `account`    | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:904

---

### revokeRole(bytes32,address)

▸ **revokeRole(bytes32,address)**(`role`: BytesLike, `account`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `role`       | BytesLike                                               |
| `account`    | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:908

---

### send

▸ **send**(`recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `recipient`  | _string_                                                |
| `amount`     | BigNumberish                                            |
| `data`       | BytesLike                                               |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:916

---

### send(address,uint256,bytes)

▸ **send(address,uint256,bytes)**(`recipient`: _string_, `amount`: BigNumberish, `data`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `recipient`  | _string_                                                |
| `amount`     | BigNumberish                                            |
| `data`       | BytesLike                                               |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:921

---

### symbol

▸ **symbol**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:930

---

### symbol()

▸ **symbol()**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:930

---

### totalSupply

▸ **totalSupply**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:934

---

### totalSupply()

▸ **totalSupply()**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:934

---

### totalSupplyAt

▸ **totalSupplyAt**(`_blockNumber`: BigNumberish, `overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name           | Type          |
| :------------- | :------------ |
| `_blockNumber` | BigNumberish  |
| `overrides?`   | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:938

---

### totalSupplyAt(uint128)

▸ **totalSupplyAt(uint128)**(`_blockNumber`: BigNumberish, `overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name           | Type          |
| :------------- | :------------ |
| `_blockNumber` | BigNumberish  |
| `overrides?`   | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:941

---

### totalSupplySnapshots

▸ **totalSupplySnapshots**(`arg0`: BigNumberish, `overrides?`: CallOverrides): _Promise_<[*BigNumber*, *BigNumber*] & { `fromBlock`: _BigNumber_ ; `value`: _BigNumber_ }\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `arg0`       | BigNumberish  |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<[*BigNumber*, *BigNumber*] & { `fromBlock`: _BigNumber_ ; `value`: _BigNumber_ }\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:948

---

### totalSupplySnapshots(uint256)

▸ **totalSupplySnapshots(uint256)**(`arg0`: BigNumberish, `overrides?`: CallOverrides): _Promise_<[*BigNumber*, *BigNumber*] & { `fromBlock`: _BigNumber_ ; `value`: _BigNumber_ }\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `arg0`       | BigNumberish  |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<[*BigNumber*, *BigNumber*] & { `fromBlock`: _BigNumber_ ; `value`: _BigNumber_ }\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:953

---

### transfer

▸ **transfer**(`recipient`: _string_, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `recipient`  | _string_                                                |
| `amount`     | BigNumberish                                            |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:962

---

### transfer(address,uint256)

▸ **transfer(address,uint256)**(`recipient`: _string_, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `recipient`  | _string_                                                |
| `amount`     | BigNumberish                                            |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:966

---

### transferFrom

▸ **transferFrom**(`holder`: _string_, `recipient`: _string_, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `holder`     | _string_                                                |
| `recipient`  | _string_                                                |
| `amount`     | BigNumberish                                            |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:974

---

### transferFrom(address,address,uint256)

▸ **transferFrom(address,address,uint256)**(`holder`: _string_, `recipient`: _string_, `amount`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `holder`     | _string_                                                |
| `recipient`  | _string_                                                |
| `amount`     | BigNumberish                                            |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprToken.d.ts:979

---

### getContractAddress

▸ `Static` **getContractAddress**(`transaction`: { `from`: _string_ ; `nonce`: BigNumberish }): _string_

#### Parameters

| Name                | Type         |
| :------------------ | :----------- |
| `transaction`       | _object_     |
| `transaction.from`  | _string_     |
| `transaction.nonce` | BigNumberish |

**Returns:** _string_

Inherited from: Contract.getContractAddress

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:100

---

### getInterface

▸ `Static` **getInterface**(`contractInterface`: ContractInterface): _Interface_

#### Parameters

| Name                | Type              |
| :------------------ | :---------------- |
| `contractInterface` | ContractInterface |

**Returns:** _Interface_

Inherited from: Contract.getInterface

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:104

---

### isIndexed

▸ `Static` **isIndexed**(`value`: _any_): value is Indexed

#### Parameters

| Name    | Type  |
| :------ | :---- |
| `value` | _any_ |

**Returns:** value is Indexed

Inherited from: Contract.isIndexed

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:110
