[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / HoprChannels

# Class: HoprChannels

## Hierarchy

- `Contract`

  ↳ **`HoprChannels`**

## Table of contents

### Constructors

- [constructor](hoprchannels.md#constructor)

### Properties

- [\_deployedPromise](hoprchannels.md#_deployedpromise)
- [\_runningEvents](hoprchannels.md#_runningevents)
- [\_wrappedEmits](hoprchannels.md#_wrappedemits)
- [address](hoprchannels.md#address)
- [callStatic](hoprchannels.md#callstatic)
- [deployTransaction](hoprchannels.md#deploytransaction)
- [estimateGas](hoprchannels.md#estimategas)
- [filters](hoprchannels.md#filters)
- [functions](hoprchannels.md#functions)
- [interface](hoprchannels.md#interface)
- [populateTransaction](hoprchannels.md#populatetransaction)
- [provider](hoprchannels.md#provider)
- [resolvedAddress](hoprchannels.md#resolvedaddress)
- [signer](hoprchannels.md#signer)

### Methods

- [FUND\_CHANNEL\_MULTI\_SIZE](hoprchannels.md#fund_channel_multi_size)
- [FUND\_CHANNEL\_MULTI\_SIZE()](hoprchannels.md#fund_channel_multi_size())
- [TOKENS\_RECIPIENT\_INTERFACE\_HASH](hoprchannels.md#tokens_recipient_interface_hash)
- [TOKENS\_RECIPIENT\_INTERFACE\_HASH()](hoprchannels.md#tokens_recipient_interface_hash())
- [\_checkRunningEvents](hoprchannels.md#_checkrunningevents)
- [\_deployed](hoprchannels.md#_deployed)
- [\_wrapEvent](hoprchannels.md#_wrapevent)
- [announce](hoprchannels.md#announce)
- [announce(bytes)](hoprchannels.md#announce(bytes))
- [attach](hoprchannels.md#attach)
- [bumpChannel](hoprchannels.md#bumpchannel)
- [bumpChannel(address,bytes32)](hoprchannels.md#bumpchannel(address,bytes32))
- [canImplementInterfaceForAddress](hoprchannels.md#canimplementinterfaceforaddress)
- [canImplementInterfaceForAddress(bytes32,address)](hoprchannels.md#canimplementinterfaceforaddress(bytes32,address))
- [channels](hoprchannels.md#channels)
- [channels(bytes32)](hoprchannels.md#channels(bytes32))
- [connect](hoprchannels.md#connect)
- [deployed](hoprchannels.md#deployed)
- [emit](hoprchannels.md#emit)
- [fallback](hoprchannels.md#fallback)
- [finalizeChannelClosure](hoprchannels.md#finalizechannelclosure)
- [finalizeChannelClosure(address)](hoprchannels.md#finalizechannelclosure(address))
- [fundChannelMulti](hoprchannels.md#fundchannelmulti)
- [fundChannelMulti(address,address,uint256,uint256)](hoprchannels.md#fundchannelmulti(address,address,uint256,uint256))
- [initiateChannelClosure](hoprchannels.md#initiatechannelclosure)
- [initiateChannelClosure(address)](hoprchannels.md#initiatechannelclosure(address))
- [listenerCount](hoprchannels.md#listenercount)
- [listeners](hoprchannels.md#listeners)
- [off](hoprchannels.md#off)
- [on](hoprchannels.md#on)
- [once](hoprchannels.md#once)
- [queryFilter](hoprchannels.md#queryfilter)
- [redeemTicket](hoprchannels.md#redeemticket)
- [redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)](hoprchannels.md#redeemticket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes))
- [removeAllListeners](hoprchannels.md#removealllisteners)
- [removeListener](hoprchannels.md#removelistener)
- [secsClosure](hoprchannels.md#secsclosure)
- [secsClosure()](hoprchannels.md#secsclosure())
- [token](hoprchannels.md#token)
- [token()](hoprchannels.md#token())
- [tokensReceived](hoprchannels.md#tokensreceived)
- [tokensReceived(address,address,address,uint256,bytes,bytes)](hoprchannels.md#tokensreceived(address,address,address,uint256,bytes,bytes))
- [getContractAddress](hoprchannels.md#getcontractaddress)
- [getInterface](hoprchannels.md#getinterface)
- [isIndexed](hoprchannels.md#isindexed)

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

packages/ethereum/types/HoprChannels.d.ts:518

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

packages/ethereum/types/HoprChannels.d.ts:730

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Announcement` | (`account`: `string`, `multiaddr`: ``null``) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`], `Object`\> |
| `ChannelUpdate` | (`source`: `string`, `destination`: `string`, `newState`: ``null``) => [`TypedEventFilter`](../interfaces/typedeventfilter.md)<[`string`, `string`, [`BigNumber`, `string`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`] & { `balance`: `BigNumber` ; `channelEpoch`: `BigNumber` ; `closureTime`: `number` ; `commitment`: `string` ; `status`: `number` ; `ticketEpoch`: `BigNumber` ; `ticketIndex`: `BigNumber`  }], `Object`\> |

#### Overrides

Contract.filters

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:679

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

packages/ethereum/types/HoprChannels.d.ts:188

___

### interface

• **interface**: `HoprChannelsInterface`

#### Overrides

Contract.interface

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:186

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

packages/ethereum/types/HoprChannels.d.ts:873

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

packages/ethereum/types/HoprChannels.d.ts:356

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

packages/ethereum/types/HoprChannels.d.ts:356

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

packages/ethereum/types/HoprChannels.d.ts:360

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

packages/ethereum/types/HoprChannels.d.ts:360

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

packages/ethereum/types/HoprChannels.d.ts:366

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

packages/ethereum/types/HoprChannels.d.ts:369

___

### attach

▸ **attach**(`addressOrName`): [`HoprChannels`](hoprchannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`HoprChannels`](hoprchannels.md)

#### Overrides

Contract.attach

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:147

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

packages/ethereum/types/HoprChannels.d.ts:376

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

packages/ethereum/types/HoprChannels.d.ts:380

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

packages/ethereum/types/HoprChannels.d.ts:388

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

packages/ethereum/types/HoprChannels.d.ts:392

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

packages/ethereum/types/HoprChannels.d.ts:400

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

packages/ethereum/types/HoprChannels.d.ts:413

___

### connect

▸ **connect**(`signerOrProvider`): [`HoprChannels`](hoprchannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`HoprChannels`](hoprchannels.md)

#### Overrides

Contract.connect

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:146

___

### deployed

▸ **deployed**(): `Promise`<[`HoprChannels`](hoprchannels.md)\>

#### Returns

`Promise`<[`HoprChannels`](hoprchannels.md)\>

#### Overrides

Contract.deployed

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:148

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

packages/ethereum/types/HoprChannels.d.ts:430

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

packages/ethereum/types/HoprChannels.d.ts:433

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

packages/ethereum/types/HoprChannels.d.ts:440

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

packages/ethereum/types/HoprChannels.d.ts:446

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

packages/ethereum/types/HoprChannels.d.ts:456

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

packages/ethereum/types/HoprChannels.d.ts:459

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

packages/ethereum/types/HoprChannels.d.ts:150

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

packages/ethereum/types/HoprChannels.d.ts:173

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprChannels`](hoprchannels.md)

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

[`HoprChannels`](hoprchannels.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:153

▸ **off**(`eventName`, `listener`): [`HoprChannels`](hoprchannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprChannels`](hoprchannels.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:174

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprChannels`](hoprchannels.md)

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

[`HoprChannels`](hoprchannels.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:157

▸ **on**(`eventName`, `listener`): [`HoprChannels`](hoprchannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprChannels`](hoprchannels.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:175

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprChannels`](hoprchannels.md)

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

[`HoprChannels`](hoprchannels.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:161

▸ **once**(`eventName`, `listener`): [`HoprChannels`](hoprchannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprChannels`](hoprchannels.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:176

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

packages/ethereum/types/HoprChannels.d.ts:180

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

packages/ethereum/types/HoprChannels.d.ts:466

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

packages/ethereum/types/HoprChannels.d.ts:476

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`HoprChannels`](hoprchannels.md)

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

[`HoprChannels`](hoprchannels.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:169

▸ **removeAllListeners**(`eventName?`): [`HoprChannels`](hoprchannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`HoprChannels`](hoprchannels.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:178

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`HoprChannels`](hoprchannels.md)

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

[`HoprChannels`](hoprchannels.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:165

▸ **removeListener**(`eventName`, `listener`): [`HoprChannels`](hoprchannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`HoprChannels`](hoprchannels.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/HoprChannels.d.ts:177

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

packages/ethereum/types/HoprChannels.d.ts:490

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

packages/ethereum/types/HoprChannels.d.ts:490

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

packages/ethereum/types/HoprChannels.d.ts:494

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

packages/ethereum/types/HoprChannels.d.ts:494

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

packages/ethereum/types/HoprChannels.d.ts:498

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

packages/ethereum/types/HoprChannels.d.ts:506

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
