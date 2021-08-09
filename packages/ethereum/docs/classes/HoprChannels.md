[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprChannels

# Class: HoprChannels

## Hierarchy

- `Contract`

  ↳ **`HoprChannels`**

## Table of contents

### Constructors

- [constructor](HoprChannels.md#constructor)

### Properties

- [\_deployedPromise](HoprChannels.md#_deployedpromise)
- [\_runningEvents](HoprChannels.md#_runningevents)
- [\_wrappedEmits](HoprChannels.md#_wrappedemits)
- [address](HoprChannels.md#address)
- [callStatic](HoprChannels.md#callstatic)
- [deployTransaction](HoprChannels.md#deploytransaction)
- [estimateGas](HoprChannels.md#estimategas)
- [filters](HoprChannels.md#filters)
- [functions](HoprChannels.md#functions)
- [interface](HoprChannels.md#interface)
- [populateTransaction](HoprChannels.md#populatetransaction)
- [provider](HoprChannels.md#provider)
- [resolvedAddress](HoprChannels.md#resolvedaddress)
- [signer](HoprChannels.md#signer)

### Methods

- [FUND\_CHANNEL\_MULTI\_SIZE](HoprChannels.md#fund_channel_multi_size)
- [FUND\_CHANNEL\_MULTI\_SIZE()](HoprChannels.md#fund_channel_multi_size())
- [TOKENS\_RECIPIENT\_INTERFACE\_HASH](HoprChannels.md#tokens_recipient_interface_hash)
- [TOKENS\_RECIPIENT\_INTERFACE\_HASH()](HoprChannels.md#tokens_recipient_interface_hash())
- [\_checkRunningEvents](HoprChannels.md#_checkrunningevents)
- [\_deployed](HoprChannels.md#_deployed)
- [\_wrapEvent](HoprChannels.md#_wrapevent)
- [announce](HoprChannels.md#announce)
- [announce(bytes)](HoprChannels.md#announce(bytes))
- [attach](HoprChannels.md#attach)
- [bumpChannel](HoprChannels.md#bumpchannel)
- [bumpChannel(address,bytes32)](HoprChannels.md#bumpchannel(address,bytes32))
- [canImplementInterfaceForAddress](HoprChannels.md#canimplementinterfaceforaddress)
- [canImplementInterfaceForAddress(bytes32,address)](HoprChannels.md#canimplementinterfaceforaddress(bytes32,address))
- [channels](HoprChannels.md#channels)
- [channels(bytes32)](HoprChannels.md#channels(bytes32))
- [connect](HoprChannels.md#connect)
- [deployed](HoprChannels.md#deployed)
- [emit](HoprChannels.md#emit)
- [fallback](HoprChannels.md#fallback)
- [finalizeChannelClosure](HoprChannels.md#finalizechannelclosure)
- [finalizeChannelClosure(address)](HoprChannels.md#finalizechannelclosure(address))
- [fundChannelMulti](HoprChannels.md#fundchannelmulti)
- [fundChannelMulti(address,address,uint256,uint256)](HoprChannels.md#fundchannelmulti(address,address,uint256,uint256))
- [initiateChannelClosure](HoprChannels.md#initiatechannelclosure)
- [initiateChannelClosure(address)](HoprChannels.md#initiatechannelclosure(address))
- [listenerCount](HoprChannels.md#listenercount)
- [listeners](HoprChannels.md#listeners)
- [multicall](HoprChannels.md#multicall)
- [multicall(bytes[])](HoprChannels.md#multicall(bytes[]))
- [off](HoprChannels.md#off)
- [on](HoprChannels.md#on)
- [once](HoprChannels.md#once)
- [queryFilter](HoprChannels.md#queryfilter)
- [redeemTicket](HoprChannels.md#redeemticket)
- [redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)](HoprChannels.md#redeemticket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes))
- [removeAllListeners](HoprChannels.md#removealllisteners)
- [removeListener](HoprChannels.md#removelistener)
- [secsClosure](HoprChannels.md#secsclosure)
- [secsClosure()](HoprChannels.md#secsclosure())
- [token](HoprChannels.md#token)
- [token()](HoprChannels.md#token())
- [tokensReceived](HoprChannels.md#tokensreceived)
- [tokensReceived(address,address,address,uint256,bytes,bytes)](HoprChannels.md#tokensreceived(address,address,address,uint256,bytes,bytes))
- [getContractAddress](HoprChannels.md#getcontractaddress)
- [getInterface](HoprChannels.md#getinterface)
- [isIndexed](HoprChannels.md#isindexed)

## Constructors

### constructor

• **new HoprChannels**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

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
| `FUND_CHANNEL_MULTI_SIZE` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `FUND_CHANNEL_MULTI_SIZE()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `announce` | (`multiaddr`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `announce(bytes)` | (`multiaddr`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `bumpChannel` | (`source`: `string`, `newCommitment`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `bumpChannel(address,bytes32)` | (`source`: `string`, `newCommitment`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `channels` | (`arg0`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `string`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`] & { `balance`: `BigNumber` ; `channelEpoch`: `BigNumber` ; `closureTime`: `number` ; `commitment`: `string` ; `status`: `number` ; `ticketEpoch`: `BigNumber` ; `ticketIndex`: `BigNumber`  }\> |
| `channels(bytes32)` | (`arg0`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `string`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`] & { `balance`: `BigNumber` ; `channelEpoch`: `BigNumber` ; `closureTime`: `number` ; `commitment`: `string` ; `status`: `number` ; `ticketEpoch`: `BigNumber` ; `ticketIndex`: `BigNumber`  }\> |
| `finalizeChannelClosure` | (`destination`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `finalizeChannelClosure(address)` | (`destination`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `fundChannelMulti` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `fundChannelMulti(address,address,uint256,uint256)` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `initiateChannelClosure` | (`destination`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `initiateChannelClosure(address)` | (`destination`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `multicall` | (`data`: `BytesLike`[], `overrides?`: `CallOverrides`) => `Promise`<`string`[]\> |
| `multicall(bytes[])` | (`data`: `BytesLike`[], `overrides?`: `CallOverrides`) => `Promise`<`string`[]\> |
| `redeemTicket` | (`source`: `string`, `nextCommitment`: `BytesLike`, `ticketEpoch`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `amount`: `BigNumberish`, `winProb`: `BigNumberish`, `signature`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | (`source`: `string`, `nextCommitment`: `BytesLike`, `ticketEpoch`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `amount`: `BigNumberish`, `winProb`: `BigNumberish`, `signature`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `secsClosure` | (`overrides?`: `CallOverrides`) => `Promise`<`number`\> |
| `secsClosure()` | (`overrides?`: `CallOverrides`) => `Promise`<`number`\> |
| `token` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `token()` | (`overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |

#### Overrides

Contract.callStatic

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:554

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
| `FUND_CHANNEL_MULTI_SIZE` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `FUND_CHANNEL_MULTI_SIZE()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `announce` | (`multiaddr`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `announce(bytes)` | (`multiaddr`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `bumpChannel` | (`source`: `string`, `newCommitment`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `bumpChannel(address,bytes32)` | (`source`: `string`, `newCommitment`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `channels` | (`arg0`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `channels(bytes32)` | (`arg0`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `finalizeChannelClosure` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `finalizeChannelClosure(address)` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `fundChannelMulti` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `fundChannelMulti(address,address,uint256,uint256)` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `initiateChannelClosure` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `initiateChannelClosure(address)` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `multicall` | (`data`: `BytesLike`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `multicall(bytes[])` | (`data`: `BytesLike`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `redeemTicket` | (`source`: `string`, `nextCommitment`: `BytesLike`, `ticketEpoch`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `amount`: `BigNumberish`, `winProb`: `BigNumberish`, `signature`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | (`source`: `string`, `nextCommitment`: `BytesLike`, `ticketEpoch`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `amount`: `BigNumberish`, `winProb`: `BigNumberish`, `signature`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `secsClosure` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `secsClosure()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `token` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `token()` | (`overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |

#### Overrides

Contract.estimateGas

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:866

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Announcement` | (`account`: `string`, `multiaddr`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`], `Object`\> |
| `ChannelBumped` | (`source`: `string`, `destination`: `string`, `newCommitment`: ``null``, `ticketEpoch`: ``null``, `channelBalance`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`, `BigNumber`, `BigNumber`], `Object`\> |
| `ChannelClosureFinalized` | (`source`: `string`, `destination`: `string`, `closureFinalizationTime`: ``null``, `channelBalance`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `number`, `BigNumber`], `Object`\> |
| `ChannelClosureInitiated` | (`source`: `string`, `destination`: `string`, `closureInitiationTime`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `number`], `Object`\> |
| `ChannelUpdate` | (`source`: `string`, `destination`: `string`, `newState`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, [`BigNumber`, `string`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`] & { `balance`: `BigNumber` ; `channelEpoch`: `BigNumber` ; `closureTime`: `number` ; `commitment`: `string` ; `status`: `number` ; `ticketEpoch`: `BigNumber` ; `ticketIndex`: `BigNumber`  }], `Object`\> |
| `TicketRedeemed` | (`source`: `string`, `destination`: `string`, `nextCommitment`: ``null``, `ticketEpoch`: ``null``, `ticketIndex`: ``null``, `proofOfRelaySecret`: ``null``, `amount`: ``null``, `winProb`: ``null``, `signature`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`, `BigNumber`, `BigNumber`, `string`, `BigNumber`, `BigNumber`, `string`], `Object`\> |
| `TokensReceived` | (`from`: `string`, `account1`: `string`, `account2`: `string`, `amount1`: ``null``, `amount2`: ``null``) => [`TypedEventFilter`](../interfaces/TypedEventFilter.md)<[`string`, `string`, `string`, `BigNumber`, `BigNumber`], `Object`\> |

#### Overrides

Contract.filters

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:722

___

### functions

• **functions**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `FUND_CHANNEL_MULTI_SIZE` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `FUND_CHANNEL_MULTI_SIZE()` | (`overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `announce` | (`multiaddr`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `announce(bytes)` | (`multiaddr`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `bumpChannel` | (`source`: `string`, `newCommitment`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `bumpChannel(address,bytes32)` | (`source`: `string`, `newCommitment`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `channels` | (`arg0`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `string`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`] & { `balance`: `BigNumber` ; `channelEpoch`: `BigNumber` ; `closureTime`: `number` ; `commitment`: `string` ; `status`: `number` ; `ticketEpoch`: `BigNumber` ; `ticketIndex`: `BigNumber`  }\> |
| `channels(bytes32)` | (`arg0`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `string`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`] & { `balance`: `BigNumber` ; `channelEpoch`: `BigNumber` ; `closureTime`: `number` ; `commitment`: `string` ; `status`: `number` ; `ticketEpoch`: `BigNumber` ; `ticketIndex`: `BigNumber`  }\> |
| `finalizeChannelClosure` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `finalizeChannelClosure(address)` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `fundChannelMulti` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `fundChannelMulti(address,address,uint256,uint256)` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `initiateChannelClosure` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `initiateChannelClosure(address)` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `multicall` | (`data`: `BytesLike`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `multicall(bytes[])` | (`data`: `BytesLike`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `redeemTicket` | (`source`: `string`, `nextCommitment`: `BytesLike`, `ticketEpoch`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `amount`: `BigNumberish`, `winProb`: `BigNumberish`, `signature`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | (`source`: `string`, `nextCommitment`: `BytesLike`, `ticketEpoch`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `amount`: `BigNumberish`, `winProb`: `BigNumberish`, `signature`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `secsClosure` | (`overrides?`: `CallOverrides`) => `Promise`<[`number`]\> |
| `secsClosure()` | (`overrides?`: `CallOverrides`) => `Promise`<[`number`]\> |
| `token` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `token()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |

#### Overrides

Contract.functions

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:204

___

### interface

• **interface**: `HoprChannelsInterface`

#### Overrides

Contract.interface

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:202

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `FUND_CHANNEL_MULTI_SIZE` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `FUND_CHANNEL_MULTI_SIZE()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `announce` | (`multiaddr`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `announce(bytes)` | (`multiaddr`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `bumpChannel` | (`source`: `string`, `newCommitment`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `bumpChannel(address,bytes32)` | (`source`: `string`, `newCommitment`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `channels` | (`arg0`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `channels(bytes32)` | (`arg0`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `finalizeChannelClosure` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `finalizeChannelClosure(address)` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `fundChannelMulti` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `fundChannelMulti(address,address,uint256,uint256)` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `initiateChannelClosure` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `initiateChannelClosure(address)` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `multicall` | (`data`: `BytesLike`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `multicall(bytes[])` | (`data`: `BytesLike`[], `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `redeemTicket` | (`source`: `string`, `nextCommitment`: `BytesLike`, `ticketEpoch`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `amount`: `BigNumberish`, `winProb`: `BigNumberish`, `signature`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | (`source`: `string`, `nextCommitment`: `BytesLike`, `ticketEpoch`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `amount`: `BigNumberish`, `winProb`: `BigNumberish`, `signature`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `secsClosure` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `secsClosure()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `token` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `token()` | (`overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |

#### Overrides

Contract.populateTransaction

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:1019

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

### FUND\_CHANNEL\_MULTI\_SIZE

▸ **FUND_CHANNEL_MULTI_SIZE**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:382

___

### FUND\_CHANNEL\_MULTI\_SIZE()

▸ **FUND_CHANNEL_MULTI_SIZE()**(`overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:384

___

### TOKENS\_RECIPIENT\_INTERFACE\_HASH

▸ **TOKENS_RECIPIENT_INTERFACE_HASH**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:386

___

### TOKENS\_RECIPIENT\_INTERFACE\_HASH()

▸ **TOKENS_RECIPIENT_INTERFACE_HASH()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:388

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

### announce

▸ **announce**(`multiaddr`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `multiaddr` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:392

___

### announce(bytes)

▸ **announce(bytes)**(`multiaddr`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `multiaddr` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:397

___

### attach

▸ **attach**(`addressOrName`): [`HoprChannels`](HoprChannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`HoprChannels`](HoprChannels.md)

#### Overrides

Contract.attach

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:163

___

### bumpChannel

▸ **bumpChannel**(`source`, `newCommitment`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `source` | `string` |
| `newCommitment` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:402

___

### bumpChannel(address,bytes32)

▸ **bumpChannel(address,bytes32)**(`source`, `newCommitment`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `source` | `string` |
| `newCommitment` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:408

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

packages/ethereum/types/HoprChannels.d.ts:414

___

### canImplementInterfaceForAddress(bytes32,address)

▸ **canImplementInterfaceForAddress(bytes32,address)**(`interfaceHash`, `account`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `interfaceHash` | `BytesLike` |
| `account` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:420

___

### channels

▸ **channels**(`arg0`, `overrides?`): `Promise`<[`BigNumber`, `string`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`] & { `balance`: `BigNumber` ; `channelEpoch`: `BigNumber` ; `closureTime`: `number` ; `commitment`: `string` ; `status`: `number` ; `ticketEpoch`: `BigNumber` ; `ticketIndex`: `BigNumber`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`BigNumber`, `string`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`] & { `balance`: `BigNumber` ; `channelEpoch`: `BigNumber` ; `closureTime`: `number` ; `commitment`: `string` ; `status`: `number` ; `ticketEpoch`: `BigNumber` ; `ticketIndex`: `BigNumber`  }\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:426

___

### channels(bytes32)

▸ **channels(bytes32)**(`arg0`, `overrides?`): `Promise`<[`BigNumber`, `string`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`] & { `balance`: `BigNumber` ; `channelEpoch`: `BigNumber` ; `closureTime`: `number` ; `commitment`: `string` ; `status`: `number` ; `ticketEpoch`: `BigNumber` ; `ticketIndex`: `BigNumber`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`BigNumber`, `string`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`] & { `balance`: `BigNumber` ; `channelEpoch`: `BigNumber` ; `closureTime`: `number` ; `commitment`: `string` ; `status`: `number` ; `ticketEpoch`: `BigNumber` ; `ticketIndex`: `BigNumber`  }\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:441

___

### connect

▸ **connect**(`signerOrProvider`): [`HoprChannels`](HoprChannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`HoprChannels`](HoprChannels.md)

#### Overrides

Contract.connect

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:162

___

### deployed

▸ **deployed**(): `Promise`<[`HoprChannels`](HoprChannels.md)\>

#### Returns

`Promise`<[`HoprChannels`](HoprChannels.md)\>

#### Overrides

Contract.deployed

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:164

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

### finalizeChannelClosure

▸ **finalizeChannelClosure**(`destination`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `destination` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:456

___

### finalizeChannelClosure(address)

▸ **finalizeChannelClosure(address)**(`destination`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `destination` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:461

___

### fundChannelMulti

▸ **fundChannelMulti**(`account1`, `account2`, `amount1`, `amount2`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account1` | `string` |
| `account2` | `string` |
| `amount1` | `BigNumberish` |
| `amount2` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:466

___

### fundChannelMulti(address,address,uint256,uint256)

▸ **fundChannelMulti(address,address,uint256,uint256)**(`account1`, `account2`, `amount1`, `amount2`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account1` | `string` |
| `account2` | `string` |
| `amount1` | `BigNumberish` |
| `amount2` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:474

___

### initiateChannelClosure

▸ **initiateChannelClosure**(`destination`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `destination` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:482

___

### initiateChannelClosure(address)

▸ **initiateChannelClosure(address)**(`destination`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `destination` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:487

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

packages/ethereum/types/HoprChannels.d.ts:166

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

packages/ethereum/types/HoprChannels.d.ts:189

___

### multicall

▸ **multicall**(`data`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `data` | `BytesLike`[] |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:492

___

### multicall(bytes[])

▸ **multicall(bytes[])**(`data`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `data` | `BytesLike`[] |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:497

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprChannels`](HoprChannels.md)

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

[`HoprChannels`](HoprChannels.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:169

▸ **off**(`eventName`, `listener`): [`HoprChannels`](HoprChannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprChannels`](HoprChannels.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:190

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprChannels`](HoprChannels.md)

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

[`HoprChannels`](HoprChannels.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:173

▸ **on**(`eventName`, `listener`): [`HoprChannels`](HoprChannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprChannels`](HoprChannels.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:191

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprChannels`](HoprChannels.md)

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

[`HoprChannels`](HoprChannels.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:177

▸ **once**(`eventName`, `listener`): [`HoprChannels`](HoprChannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprChannels`](HoprChannels.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:192

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

packages/ethereum/types/HoprChannels.d.ts:196

___

### redeemTicket

▸ **redeemTicket**(`source`, `nextCommitment`, `ticketEpoch`, `ticketIndex`, `proofOfRelaySecret`, `amount`, `winProb`, `signature`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `source` | `string` |
| `nextCommitment` | `BytesLike` |
| `ticketEpoch` | `BigNumberish` |
| `ticketIndex` | `BigNumberish` |
| `proofOfRelaySecret` | `BytesLike` |
| `amount` | `BigNumberish` |
| `winProb` | `BigNumberish` |
| `signature` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:502

___

### redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)

▸ **redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)**(`source`, `nextCommitment`, `ticketEpoch`, `ticketIndex`, `proofOfRelaySecret`, `amount`, `winProb`, `signature`, `overrides?`): `Promise`<`ContractTransaction`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `source` | `string` |
| `nextCommitment` | `BytesLike` |
| `ticketEpoch` | `BigNumberish` |
| `ticketIndex` | `BigNumberish` |
| `proofOfRelaySecret` | `BytesLike` |
| `amount` | `BigNumberish` |
| `winProb` | `BigNumberish` |
| `signature` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  } |

#### Returns

`Promise`<`ContractTransaction`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:514

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`HoprChannels`](HoprChannels.md)

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

[`HoprChannels`](HoprChannels.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:185

▸ **removeAllListeners**(`eventName?`): [`HoprChannels`](HoprChannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`HoprChannels`](HoprChannels.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:194

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprChannels`](HoprChannels.md)

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

[`HoprChannels`](HoprChannels.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:181

▸ **removeListener**(`eventName`, `listener`): [`HoprChannels`](HoprChannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprChannels`](HoprChannels.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:193

___

### secsClosure

▸ **secsClosure**(`overrides?`): `Promise`<`number`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`number`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:526

___

### secsClosure()

▸ **secsClosure()**(`overrides?`): `Promise`<`number`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`number`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:528

___

### token

▸ **token**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:530

___

### token()

▸ **token()**(`overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:532

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

packages/ethereum/types/HoprChannels.d.ts:534

___

### tokensReceived(address,address,address,uint256,bytes,bytes)

▸ **tokensReceived(address,address,address,uint256,bytes,bytes)**(`operator`, `from`, `to`, `amount`, `userData`, `operatorData`, `overrides?`): `Promise`<`ContractTransaction`\>

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

packages/ethereum/types/HoprChannels.d.ts:544

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
