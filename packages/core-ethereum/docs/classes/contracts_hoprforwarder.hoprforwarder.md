[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprForwarder](../modules/contracts_hoprforwarder.md) / HoprForwarder

# Class: HoprForwarder

[contracts/HoprForwarder](../modules/contracts_hoprforwarder.md).HoprForwarder

## Hierarchy

- _Contract_

  ↳ **HoprForwarder**

## Table of contents

### Constructors

- [constructor](contracts_hoprforwarder.hoprforwarder.md#constructor)

### Properties

- [\_deployedPromise](contracts_hoprforwarder.hoprforwarder.md#_deployedpromise)
- [\_runningEvents](contracts_hoprforwarder.hoprforwarder.md#_runningevents)
- [\_wrappedEmits](contracts_hoprforwarder.hoprforwarder.md#_wrappedemits)
- [address](contracts_hoprforwarder.hoprforwarder.md#address)
- [callStatic](contracts_hoprforwarder.hoprforwarder.md#callstatic)
- [deployTransaction](contracts_hoprforwarder.hoprforwarder.md#deploytransaction)
- [estimateGas](contracts_hoprforwarder.hoprforwarder.md#estimategas)
- [filters](contracts_hoprforwarder.hoprforwarder.md#filters)
- [functions](contracts_hoprforwarder.hoprforwarder.md#functions)
- [interface](contracts_hoprforwarder.hoprforwarder.md#interface)
- [populateTransaction](contracts_hoprforwarder.hoprforwarder.md#populatetransaction)
- [provider](contracts_hoprforwarder.hoprforwarder.md#provider)
- [resolvedAddress](contracts_hoprforwarder.hoprforwarder.md#resolvedaddress)
- [signer](contracts_hoprforwarder.hoprforwarder.md#signer)

### Methods

- [ERC1820_REGISTRY](contracts_hoprforwarder.hoprforwarder.md#erc1820_registry)
- [ERC1820_REGISTRY()](<contracts_hoprforwarder.hoprforwarder.md#erc1820_registry()>)
- [HOPR_TOKEN](contracts_hoprforwarder.hoprforwarder.md#hopr_token)
- [HOPR_TOKEN()](<contracts_hoprforwarder.hoprforwarder.md#hopr_token()>)
- [MULTISIG](contracts_hoprforwarder.hoprforwarder.md#multisig)
- [MULTISIG()](<contracts_hoprforwarder.hoprforwarder.md#multisig()>)
- [TOKENS_RECIPIENT_INTERFACE_HASH](contracts_hoprforwarder.hoprforwarder.md#tokens_recipient_interface_hash)
- [TOKENS_RECIPIENT_INTERFACE_HASH()](<contracts_hoprforwarder.hoprforwarder.md#tokens_recipient_interface_hash()>)
- [\_checkRunningEvents](contracts_hoprforwarder.hoprforwarder.md#_checkrunningevents)
- [\_deployed](contracts_hoprforwarder.hoprforwarder.md#_deployed)
- [\_wrapEvent](contracts_hoprforwarder.hoprforwarder.md#_wrapevent)
- [attach](contracts_hoprforwarder.hoprforwarder.md#attach)
- [connect](contracts_hoprforwarder.hoprforwarder.md#connect)
- [deployed](contracts_hoprforwarder.hoprforwarder.md#deployed)
- [emit](contracts_hoprforwarder.hoprforwarder.md#emit)
- [fallback](contracts_hoprforwarder.hoprforwarder.md#fallback)
- [listenerCount](contracts_hoprforwarder.hoprforwarder.md#listenercount)
- [listeners](contracts_hoprforwarder.hoprforwarder.md#listeners)
- [off](contracts_hoprforwarder.hoprforwarder.md#off)
- [on](contracts_hoprforwarder.hoprforwarder.md#on)
- [once](contracts_hoprforwarder.hoprforwarder.md#once)
- [queryFilter](contracts_hoprforwarder.hoprforwarder.md#queryfilter)
- [recoverTokens](contracts_hoprforwarder.hoprforwarder.md#recovertokens)
- [recoverTokens(address)](<contracts_hoprforwarder.hoprforwarder.md#recovertokens(address)>)
- [removeAllListeners](contracts_hoprforwarder.hoprforwarder.md#removealllisteners)
- [removeListener](contracts_hoprforwarder.hoprforwarder.md#removelistener)
- [tokensReceived](contracts_hoprforwarder.hoprforwarder.md#tokensreceived)
- [tokensReceived(address,address,address,uint256,bytes,bytes)](<contracts_hoprforwarder.hoprforwarder.md#tokensreceived(address,address,address,uint256,bytes,bytes)>)
- [getContractAddress](contracts_hoprforwarder.hoprforwarder.md#getcontractaddress)
- [getInterface](contracts_hoprforwarder.hoprforwarder.md#getinterface)
- [isIndexed](contracts_hoprforwarder.hoprforwarder.md#isindexed)

## Constructors

### constructor

\+ **new HoprForwarder**(`addressOrName`: _string_, `contractInterface`: ContractInterface, `signerOrProvider?`: _Provider_ \| _Signer_): [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

#### Parameters

| Name                | Type                   |
| :------------------ | :--------------------- |
| `addressOrName`     | _string_               |
| `contractInterface` | ContractInterface      |
| `signerOrProvider?` | _Provider_ \| _Signer_ |

**Returns:** [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

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

| Name                                                          | Type                                                                                                                                                                                |
| :------------------------------------------------------------ | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ERC1820_REGISTRY`                                            | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                 |
| `ERC1820_REGISTRY()`                                          | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                 |
| `HOPR_TOKEN`                                                  | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                 |
| `HOPR_TOKEN()`                                                | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                 |
| `MULTISIG`                                                    | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                 |
| `MULTISIG()`                                                  | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                 |
| `TOKENS_RECIPIENT_INTERFACE_HASH`                             | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                 |
| `TOKENS_RECIPIENT_INTERFACE_HASH()`                           | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                 |
| `recoverTokens`                                               | (`token`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                |
| `recoverTokens(address)`                                      | (`token`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                |
| `tokensReceived`                                              | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\> |

Overrides: Contract.callStatic

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:219

---

### deployTransaction

• `Readonly` **deployTransaction**: TransactionResponse

Inherited from: Contract.deployTransaction

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:91

---

### estimateGas

• **estimateGas**: _object_

#### Type declaration

| Name                                                          | Type                                                                                                                                                                                                                               |
| :------------------------------------------------------------ | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ERC1820_REGISTRY`                                            | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |
| `ERC1820_REGISTRY()`                                          | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |
| `HOPR_TOKEN`                                                  | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |
| `HOPR_TOKEN()`                                                | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |
| `MULTISIG`                                                    | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |
| `MULTISIG()`                                                  | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |
| `TOKENS_RECIPIENT_INTERFACE_HASH`                             | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |
| `TOKENS_RECIPIENT_INTERFACE_HASH()`                           | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                             |
| `recoverTokens`                                               | (`token`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                                |
| `recoverTokens(address)`                                      | (`token`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                                |
| `tokensReceived`                                              | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\> |

Overrides: Contract.estimateGas

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:268

---

### filters

• **filters**: _object_

#### Type declaration

Overrides: Contract.filters

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:266

---

### functions

• **functions**: _object_

#### Type declaration

| Name                                                          | Type                                                                                                                                                                                                                                         |
| :------------------------------------------------------------ | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ERC1820_REGISTRY`                                            | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                      |
| `ERC1820_REGISTRY()`                                          | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                      |
| `HOPR_TOKEN`                                                  | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                      |
| `HOPR_TOKEN()`                                                | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                      |
| `MULTISIG`                                                    | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                      |
| `MULTISIG()`                                                  | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                      |
| `TOKENS_RECIPIENT_INTERFACE_HASH`                             | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                      |
| `TOKENS_RECIPIENT_INTERFACE_HASH()`                           | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                      |
| `recoverTokens`                                               | (`token`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                |
| `recoverTokens(address)`                                      | (`token`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                |
| `tokensReceived`                                              | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\> |

Overrides: Contract.functions

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:119

---

### interface

• **interface**: [_HoprForwarderInterface_](../interfaces/contracts_hoprforwarder.hoprforwarderinterface.md)

Overrides: Contract.interface

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:117

---

### populateTransaction

• **populateTransaction**: _object_

#### Type declaration

| Name                                                          | Type                                                                                                                                                                                                                                          |
| :------------------------------------------------------------ | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ERC1820_REGISTRY`                                            | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |
| `ERC1820_REGISTRY()`                                          | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |
| `HOPR_TOKEN`                                                  | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |
| `HOPR_TOKEN()`                                                | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |
| `MULTISIG`                                                    | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |
| `MULTISIG()`                                                  | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |
| `TOKENS_RECIPIENT_INTERFACE_HASH`                             | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |
| `TOKENS_RECIPIENT_INTERFACE_HASH()`                           | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                             |
| `recoverTokens`                                               | (`token`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                                |
| `recoverTokens(address)`                                      | (`token`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                                |
| `tokensReceived`                                              | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\> |

Overrides: Contract.populateTransaction

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:320

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

### ERC1820_REGISTRY

▸ **ERC1820_REGISTRY**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:171

---

### ERC1820_REGISTRY()

▸ **ERC1820_REGISTRY()**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:171

---

### HOPR_TOKEN

▸ **HOPR_TOKEN**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:175

---

### HOPR_TOKEN()

▸ **HOPR_TOKEN()**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:175

---

### MULTISIG

▸ **MULTISIG**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:179

---

### MULTISIG()

▸ **MULTISIG()**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:179

---

### TOKENS_RECIPIENT_INTERFACE_HASH

▸ **TOKENS_RECIPIENT_INTERFACE_HASH**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:183

---

### TOKENS_RECIPIENT_INTERFACE_HASH()

▸ **TOKENS_RECIPIENT_INTERFACE_HASH()**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:183

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

### attach

▸ **attach**(`addressOrName`: _string_): [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

#### Parameters

| Name            | Type     |
| :-------------- | :------- |
| `addressOrName` | _string_ |

**Returns:** [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

Overrides: Contract.attach

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:78

---

### connect

▸ **connect**(`signerOrProvider`: _string_ \| _Provider_ \| _Signer_): [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

#### Parameters

| Name               | Type                               |
| :----------------- | :--------------------------------- |
| `signerOrProvider` | _string_ \| _Provider_ \| _Signer_ |

**Returns:** [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

Overrides: Contract.connect

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:77

---

### deployed

▸ **deployed**(): _Promise_<[_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)\>

**Returns:** _Promise_<[_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)\>

Overrides: Contract.deployed

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:79

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

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:81

▸ **listeners**(`eventName?`: _string_): Listener[]

#### Parameters

| Name         | Type     |
| :----------- | :------- |
| `eventName?` | _string_ |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:104

---

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

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

**Returns:** [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

Overrides: Contract.off

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:84

▸ **off**(`eventName`: _string_, `listener`: Listener): [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

Overrides: Contract.off

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:105

---

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

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

**Returns:** [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

Overrides: Contract.on

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:88

▸ **on**(`eventName`: _string_, `listener`: Listener): [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

Overrides: Contract.on

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:106

---

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

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

**Returns:** [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

Overrides: Contract.once

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:92

▸ **once**(`eventName`: _string_, `listener`: Listener): [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

Overrides: Contract.once

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:107

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

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:111

---

### recoverTokens

▸ **recoverTokens**(`token`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `token`      | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:189

---

### recoverTokens(address)

▸ **recoverTokens(address)**(`token`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `token`      | _string_                                                |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:192

---

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name          | Type                                                                                                        |
| :------------ | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

Overrides: Contract.removeAllListeners

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:100

▸ **removeAllListeners**(`eventName?`: _string_): [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

#### Parameters

| Name         | Type     |
| :----------- | :------- |
| `eventName?` | _string_ |

**Returns:** [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

Overrides: Contract.removeAllListeners

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:109

---

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

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

**Returns:** [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

Overrides: Contract.removeListener

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:96

▸ **removeListener**(`eventName`: _string_, `listener`: Listener): [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprForwarder_](contracts_hoprforwarder.hoprforwarder.md)

Overrides: Contract.removeListener

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:108

---

### tokensReceived

▸ **tokensReceived**(`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `operator`     | _string_                                                |
| `from`         | _string_                                                |
| `to`           | _string_                                                |
| `amount`       | BigNumberish                                            |
| `userData`     | BytesLike                                               |
| `operatorData` | BytesLike                                               |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:199

---

### tokensReceived(address,address,address,uint256,bytes,bytes)

▸ **tokensReceived(address,address,address,uint256,bytes,bytes)**(`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `operator`     | _string_                                                |
| `from`         | _string_                                                |
| `to`           | _string_                                                |
| `amount`       | BigNumberish                                            |
| `userData`     | BytesLike                                               |
| `operatorData` | BytesLike                                               |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprForwarder.d.ts:207

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
