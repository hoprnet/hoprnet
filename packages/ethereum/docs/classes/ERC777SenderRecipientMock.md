[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ERC777SenderRecipientMock

# Class: ERC777SenderRecipientMock

## Hierarchy

- `BaseContract`

  ↳ **`ERC777SenderRecipientMock`**

## Table of contents

### Constructors

- [constructor](ERC777SenderRecipientMock.md#constructor)

### Properties

- [\_deployedPromise](ERC777SenderRecipientMock.md#_deployedpromise)
- [\_runningEvents](ERC777SenderRecipientMock.md#_runningevents)
- [\_wrappedEmits](ERC777SenderRecipientMock.md#_wrappedemits)
- [address](ERC777SenderRecipientMock.md#address)
- [callStatic](ERC777SenderRecipientMock.md#callstatic)
- [deployTransaction](ERC777SenderRecipientMock.md#deploytransaction)
- [estimateGas](ERC777SenderRecipientMock.md#estimategas)
- [filters](ERC777SenderRecipientMock.md#filters)
- [functions](ERC777SenderRecipientMock.md#functions)
- [interface](ERC777SenderRecipientMock.md#interface)
- [populateTransaction](ERC777SenderRecipientMock.md#populatetransaction)
- [provider](ERC777SenderRecipientMock.md#provider)
- [resolvedAddress](ERC777SenderRecipientMock.md#resolvedaddress)
- [signer](ERC777SenderRecipientMock.md#signer)

### Methods

- [\_checkRunningEvents](ERC777SenderRecipientMock.md#_checkrunningevents)
- [\_deployed](ERC777SenderRecipientMock.md#_deployed)
- [\_wrapEvent](ERC777SenderRecipientMock.md#_wrapevent)
- [attach](ERC777SenderRecipientMock.md#attach)
- [burn](ERC777SenderRecipientMock.md#burn)
- [canImplementInterfaceForAddress](ERC777SenderRecipientMock.md#canimplementinterfaceforaddress)
- [connect](ERC777SenderRecipientMock.md#connect)
- [deployed](ERC777SenderRecipientMock.md#deployed)
- [emit](ERC777SenderRecipientMock.md#emit)
- [fallback](ERC777SenderRecipientMock.md#fallback)
- [listenerCount](ERC777SenderRecipientMock.md#listenercount)
- [listeners](ERC777SenderRecipientMock.md#listeners)
- [off](ERC777SenderRecipientMock.md#off)
- [on](ERC777SenderRecipientMock.md#on)
- [once](ERC777SenderRecipientMock.md#once)
- [queryFilter](ERC777SenderRecipientMock.md#queryfilter)
- [recipientFor](ERC777SenderRecipientMock.md#recipientfor)
- [registerRecipient](ERC777SenderRecipientMock.md#registerrecipient)
- [registerSender](ERC777SenderRecipientMock.md#registersender)
- [removeAllListeners](ERC777SenderRecipientMock.md#removealllisteners)
- [removeListener](ERC777SenderRecipientMock.md#removelistener)
- [send](ERC777SenderRecipientMock.md#send)
- [senderFor](ERC777SenderRecipientMock.md#senderfor)
- [setShouldRevertReceive](ERC777SenderRecipientMock.md#setshouldrevertreceive)
- [setShouldRevertSend](ERC777SenderRecipientMock.md#setshouldrevertsend)
- [tokensReceived](ERC777SenderRecipientMock.md#tokensreceived)
- [tokensToSend](ERC777SenderRecipientMock.md#tokenstosend)
- [getContractAddress](ERC777SenderRecipientMock.md#getcontractaddress)
- [getInterface](ERC777SenderRecipientMock.md#getinterface)
- [isIndexed](ERC777SenderRecipientMock.md#isindexed)

## Constructors

### constructor

• **new ERC777SenderRecipientMock**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

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
| `burn` | (`token`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `recipientFor` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `registerRecipient` | (`recipient`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `registerSender` | (`sender`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `send` | (`token`: `string`, `to`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `senderFor` | (`account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `setShouldRevertReceive` | (`shouldRevert`: `boolean`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `setShouldRevertSend` | (`shouldRevert`: `boolean`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `tokensToSend` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |

#### Overrides

BaseContract.callStatic

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:359

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
| `burn` | (`token`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `recipientFor` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `registerRecipient` | (`recipient`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `registerSender` | (`sender`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `send` | (`token`: `string`, `to`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `senderFor` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `setShouldRevertReceive` | (`shouldRevert`: `boolean`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `setShouldRevertSend` | (`shouldRevert`: `boolean`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `tokensToSend` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

BaseContract.estimateGas

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:565

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `TokensReceivedCalled` | (`operator?`: ``null``, `from?`: ``null``, `to?`: ``null``, `amount?`: ``null``, `data?`: ``null``, `operatorData?`: ``null``, `token?`: ``null``, `fromBalance?`: ``null``, `toBalance?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`, `BigNumber`, `string`, `string`, `string`, `BigNumber`, `BigNumber`], { `amount`: `BigNumber` ; `data`: `string` ; `from`: `string` ; `fromBalance`: `BigNumber` ; `operator`: `string` ; `operatorData`: `string` ; `to`: `string` ; `toBalance`: `BigNumber` ; `token`: `string`  }\> |
| `TokensReceivedCalled(address,address,address,uint256,bytes,bytes,address,uint256,uint256)` | (`operator?`: ``null``, `from?`: ``null``, `to?`: ``null``, `amount?`: ``null``, `data?`: ``null``, `operatorData?`: ``null``, `token?`: ``null``, `fromBalance?`: ``null``, `toBalance?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`, `BigNumber`, `string`, `string`, `string`, `BigNumber`, `BigNumber`], { `amount`: `BigNumber` ; `data`: `string` ; `from`: `string` ; `fromBalance`: `BigNumber` ; `operator`: `string` ; `operatorData`: `string` ; `to`: `string` ; `toBalance`: `BigNumber` ; `token`: `string`  }\> |
| `TokensToSendCalled` | (`operator?`: ``null``, `from?`: ``null``, `to?`: ``null``, `amount?`: ``null``, `data?`: ``null``, `operatorData?`: ``null``, `token?`: ``null``, `fromBalance?`: ``null``, `toBalance?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`, `BigNumber`, `string`, `string`, `string`, `BigNumber`, `BigNumber`], { `amount`: `BigNumber` ; `data`: `string` ; `from`: `string` ; `fromBalance`: `BigNumber` ; `operator`: `string` ; `operatorData`: `string` ; `to`: `string` ; `toBalance`: `BigNumber` ; `token`: `string`  }\> |
| `TokensToSendCalled(address,address,address,uint256,bytes,bytes,address,uint256,uint256)` | (`operator?`: ``null``, `from?`: ``null``, `to?`: ``null``, `amount?`: ``null``, `data?`: ``null``, `operatorData?`: ``null``, `token?`: ``null``, `fromBalance?`: ``null``, `toBalance?`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`, `BigNumber`, `string`, `string`, `string`, `BigNumber`, `BigNumber`], { `amount`: `BigNumber` ; `data`: `string` ; `from`: `string` ; `fromBalance`: `BigNumber` ; `operator`: `string` ; `operatorData`: `string` ; `to`: `string` ; `toBalance`: `BigNumber` ; `token`: `string`  }\> |

#### Overrides

BaseContract.filters

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:423

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `burn` | (`token`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `recipientFor` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `registerRecipient` | (`recipient`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `registerSender` | (`sender`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `send` | (`token`: `string`, `to`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `senderFor` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `setShouldRevertReceive` | (`shouldRevert`: `boolean`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `setShouldRevertSend` | (`shouldRevert`: `boolean`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `tokensToSend` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

BaseContract.functions

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:215

___

### interface

• **interface**: `ERC777SenderRecipientMockInterface`

#### Overrides

BaseContract.interface

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:213

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `burn` | (`token`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `recipientFor` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `registerRecipient` | (`recipient`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `registerSender` | (`sender`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `send` | (`token`: `string`, `to`: `string`, `amount`: `BigNumberish`, `data`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `senderFor` | (`account`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `setShouldRevertReceive` | (`shouldRevert`: `boolean`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `setShouldRevertSend` | (`shouldRevert`: `boolean`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `tokensToSend` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

BaseContract.populateTransaction

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:638

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

▸ **attach**(`addressOrName`): [`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Overrides

BaseContract.attach

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:174

___

### burn

▸ **burn**(`token`, `amount`, `data`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `token` | `string` |
| `amount` | `BigNumberish` |
| `data` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:288

___

### canImplementInterfaceForAddress

▸ **canImplementInterfaceForAddress**(`interfaceHash`, `account`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `interfaceHash` | `BytesLike` |
| `account` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:295

___

### connect

▸ **connect**(`signerOrProvider`): [`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Overrides

BaseContract.connect

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:173

___

### deployed

▸ **deployed**(): `Promise`<[`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)\>

#### Returns

`Promise`<[`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)\>

#### Overrides

BaseContract.deployed

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:175

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

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:177

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

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:200

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

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

[`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:180

▸ **off**(`eventName`, `listener`): [`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Overrides

BaseContract.off

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:201

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

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

[`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:184

▸ **on**(`eventName`, `listener`): [`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Overrides

BaseContract.on

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:202

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

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

[`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:188

▸ **once**(`eventName`, `listener`): [`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Overrides

BaseContract.once

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:203

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

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:207

___

### recipientFor

▸ **recipientFor**(`account`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:301

___

### registerRecipient

▸ **registerRecipient**(`recipient`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `recipient` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:306

___

### registerSender

▸ **registerSender**(`sender`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `sender` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:311

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

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

[`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:196

▸ **removeAllListeners**(`eventName?`): [`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Overrides

BaseContract.removeAllListeners

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:205

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

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

[`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:192

▸ **removeListener**(`eventName`, `listener`): [`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ERC777SenderRecipientMock`](ERC777SenderRecipientMock.md)

#### Overrides

BaseContract.removeListener

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:204

___

### send

▸ **send**(`token`, `to`, `amount`, `data`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `token` | `string` |
| `to` | `string` |
| `amount` | `BigNumberish` |
| `data` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:316

___

### senderFor

▸ **senderFor**(`account`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:324

___

### setShouldRevertReceive

▸ **setShouldRevertReceive**(`shouldRevert`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `shouldRevert` | `boolean` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:329

___

### setShouldRevertSend

▸ **setShouldRevertSend**(`shouldRevert`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `shouldRevert` | `boolean` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:334

___

### tokensReceived

▸ **tokensReceived**(`operator`, `from`, `to`, `amount`, `userData`, `operatorData`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | `string` |
| `from` | `string` |
| `to` | `string` |
| `amount` | `BigNumberish` |
| `userData` | `BytesLike` |
| `operatorData` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:339

___

### tokensToSend

▸ **tokensToSend**(`operator`, `from`, `to`, `amount`, `userData`, `operatorData`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | `string` |
| `from` | `string` |
| `to` | `string` |
| `amount` | `BigNumberish` |
| `userData` | `BytesLike` |
| `operatorData` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/src/types/ERC777SenderRecipientMock.d.ts:349

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
