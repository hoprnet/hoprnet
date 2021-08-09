[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ChannelsMock

# Class: ChannelsMock

## Hierarchy

- `Contract`

  ↳ **`ChannelsMock`**

## Table of contents

### Constructors

- [constructor](ChannelsMock.md#constructor)

### Properties

- [\_deployedPromise](ChannelsMock.md#_deployedpromise)
- [\_runningEvents](ChannelsMock.md#_runningevents)
- [\_wrappedEmits](ChannelsMock.md#_wrappedemits)
- [address](ChannelsMock.md#address)
- [callStatic](ChannelsMock.md#callstatic)
- [deployTransaction](ChannelsMock.md#deploytransaction)
- [estimateGas](ChannelsMock.md#estimategas)
- [filters](ChannelsMock.md#filters)
- [functions](ChannelsMock.md#functions)
- [interface](ChannelsMock.md#interface)
- [populateTransaction](ChannelsMock.md#populatetransaction)
- [provider](ChannelsMock.md#provider)
- [resolvedAddress](ChannelsMock.md#resolvedaddress)
- [signer](ChannelsMock.md#signer)

### Methods

- [FUND\_CHANNEL\_MULTI\_SIZE](ChannelsMock.md#fund_channel_multi_size)
- [FUND\_CHANNEL\_MULTI\_SIZE()](ChannelsMock.md#fund_channel_multi_size())
- [TOKENS\_RECIPIENT\_INTERFACE\_HASH](ChannelsMock.md#tokens_recipient_interface_hash)
- [TOKENS\_RECIPIENT\_INTERFACE\_HASH()](ChannelsMock.md#tokens_recipient_interface_hash())
- [\_checkRunningEvents](ChannelsMock.md#_checkrunningevents)
- [\_deployed](ChannelsMock.md#_deployed)
- [\_wrapEvent](ChannelsMock.md#_wrapevent)
- [announce](ChannelsMock.md#announce)
- [announce(bytes)](ChannelsMock.md#announce(bytes))
- [attach](ChannelsMock.md#attach)
- [bumpChannel](ChannelsMock.md#bumpchannel)
- [bumpChannel(address,bytes32)](ChannelsMock.md#bumpchannel(address,bytes32))
- [canImplementInterfaceForAddress](ChannelsMock.md#canimplementinterfaceforaddress)
- [canImplementInterfaceForAddress(bytes32,address)](ChannelsMock.md#canimplementinterfaceforaddress(bytes32,address))
- [channels](ChannelsMock.md#channels)
- [channels(bytes32)](ChannelsMock.md#channels(bytes32))
- [computeChallengeInternal](ChannelsMock.md#computechallengeinternal)
- [computeChallengeInternal(bytes32)](ChannelsMock.md#computechallengeinternal(bytes32))
- [connect](ChannelsMock.md#connect)
- [deployed](ChannelsMock.md#deployed)
- [emit](ChannelsMock.md#emit)
- [fallback](ChannelsMock.md#fallback)
- [finalizeChannelClosure](ChannelsMock.md#finalizechannelclosure)
- [finalizeChannelClosure(address)](ChannelsMock.md#finalizechannelclosure(address))
- [fundChannelMulti](ChannelsMock.md#fundchannelmulti)
- [fundChannelMulti(address,address,uint256,uint256)](ChannelsMock.md#fundchannelmulti(address,address,uint256,uint256))
- [getChannelIdInternal](ChannelsMock.md#getchannelidinternal)
- [getChannelIdInternal(address,address)](ChannelsMock.md#getchannelidinternal(address,address))
- [getEncodedTicketInternal](ChannelsMock.md#getencodedticketinternal)
- [getEncodedTicketInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)](ChannelsMock.md#getencodedticketinternal(address,uint256,bytes32,uint256,uint256,uint256,uint256))
- [getTicketHashInternal](ChannelsMock.md#gettickethashinternal)
- [getTicketHashInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)](ChannelsMock.md#gettickethashinternal(address,uint256,bytes32,uint256,uint256,uint256,uint256))
- [getTicketLuckInternal](ChannelsMock.md#getticketluckinternal)
- [getTicketLuckInternal(bytes32,bytes32,bytes32)](ChannelsMock.md#getticketluckinternal(bytes32,bytes32,bytes32))
- [initiateChannelClosure](ChannelsMock.md#initiatechannelclosure)
- [initiateChannelClosure(address)](ChannelsMock.md#initiatechannelclosure(address))
- [listenerCount](ChannelsMock.md#listenercount)
- [listeners](ChannelsMock.md#listeners)
- [multicall](ChannelsMock.md#multicall)
- [multicall(bytes[])](ChannelsMock.md#multicall(bytes[]))
- [off](ChannelsMock.md#off)
- [on](ChannelsMock.md#on)
- [once](ChannelsMock.md#once)
- [queryFilter](ChannelsMock.md#queryfilter)
- [redeemTicket](ChannelsMock.md#redeemticket)
- [redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)](ChannelsMock.md#redeemticket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes))
- [removeAllListeners](ChannelsMock.md#removealllisteners)
- [removeListener](ChannelsMock.md#removelistener)
- [secsClosure](ChannelsMock.md#secsclosure)
- [secsClosure()](ChannelsMock.md#secsclosure())
- [token](ChannelsMock.md#token)
- [token()](ChannelsMock.md#token())
- [tokensReceived](ChannelsMock.md#tokensreceived)
- [tokensReceived(address,address,address,uint256,bytes,bytes)](ChannelsMock.md#tokensreceived(address,address,address,uint256,bytes,bytes))
- [getContractAddress](ChannelsMock.md#getcontractaddress)
- [getInterface](ChannelsMock.md#getinterface)
- [isIndexed](ChannelsMock.md#isindexed)

## Constructors

### constructor

• **new ChannelsMock**(`addressOrName`, `contractInterface`, `signerOrProvider?`)

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
| `computeChallengeInternal` | (`response`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `computeChallengeInternal(bytes32)` | (`response`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `finalizeChannelClosure` | (`destination`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `finalizeChannelClosure(address)` | (`destination`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `fundChannelMulti` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `fundChannelMulti(address,address,uint256,uint256)` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`void`\> |
| `getChannelIdInternal` | (`partyA`: `string`, `partyB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `getChannelIdInternal(address,address)` | (`partyA`: `string`, `partyB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `getEncodedTicketInternal` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `getEncodedTicketInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `getTicketHashInternal` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `getTicketHashInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`string`\> |
| `getTicketLuckInternal` | (`ticketHash`: `BytesLike`, `secretPreImage`: `BytesLike`, `proofOfRelaySecret`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getTicketLuckInternal(bytes32,bytes32,bytes32)` | (`ticketHash`: `BytesLike`, `secretPreImage`: `BytesLike`, `proofOfRelaySecret`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
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

packages/ethereum/types/ChannelsMock.d.ts:775

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
| `computeChallengeInternal` | (`response`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `computeChallengeInternal(bytes32)` | (`response`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `finalizeChannelClosure` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `finalizeChannelClosure(address)` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `fundChannelMulti` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `fundChannelMulti(address,address,uint256,uint256)` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`BigNumber`\> |
| `getChannelIdInternal` | (`partyA`: `string`, `partyB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getChannelIdInternal(address,address)` | (`partyA`: `string`, `partyB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getEncodedTicketInternal` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getEncodedTicketInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getTicketHashInternal` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getTicketHashInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getTicketLuckInternal` | (`ticketHash`: `BytesLike`, `secretPreImage`: `BytesLike`, `proofOfRelaySecret`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
| `getTicketLuckInternal(bytes32,bytes32,bytes32)` | (`ticketHash`: `BytesLike`, `secretPreImage`: `BytesLike`, `proofOfRelaySecret`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`BigNumber`\> |
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

packages/ethereum/types/ChannelsMock.d.ts:1167

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

packages/ethereum/types/ChannelsMock.d.ts:1023

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
| `computeChallengeInternal` | (`response`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `computeChallengeInternal(bytes32)` | (`response`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `finalizeChannelClosure` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `finalizeChannelClosure(address)` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `fundChannelMulti` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `fundChannelMulti(address,address,uint256,uint256)` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`ContractTransaction`\> |
| `getChannelIdInternal` | (`partyA`: `string`, `partyB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getChannelIdInternal(address,address)` | (`partyA`: `string`, `partyB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getEncodedTicketInternal` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getEncodedTicketInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getTicketHashInternal` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getTicketHashInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getTicketLuckInternal` | (`ticketHash`: `BytesLike`, `secretPreImage`: `BytesLike`, `proofOfRelaySecret`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `getTicketLuckInternal(bytes32,bytes32,bytes32)` | (`ticketHash`: `BytesLike`, `secretPreImage`: `BytesLike`, `proofOfRelaySecret`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
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

packages/ethereum/types/ChannelsMock.d.ts:265

___

### interface

• **interface**: `ChannelsMockInterface`

#### Overrides

Contract.interface

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:263

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
| `computeChallengeInternal` | (`response`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `computeChallengeInternal(bytes32)` | (`response`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `finalizeChannelClosure` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `finalizeChannelClosure(address)` | (`destination`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `fundChannelMulti` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `fundChannelMulti(address,address,uint256,uint256)` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<`string`\>  }) => `Promise`<`PopulatedTransaction`\> |
| `getChannelIdInternal` | (`partyA`: `string`, `partyB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getChannelIdInternal(address,address)` | (`partyA`: `string`, `partyB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getEncodedTicketInternal` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getEncodedTicketInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getTicketHashInternal` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getTicketHashInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getTicketLuckInternal` | (`ticketHash`: `BytesLike`, `secretPreImage`: `BytesLike`, `proofOfRelaySecret`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
| `getTicketLuckInternal(bytes32,bytes32,bytes32)` | (`ticketHash`: `BytesLike`, `secretPreImage`: `BytesLike`, `proofOfRelaySecret`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<`PopulatedTransaction`\> |
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

packages/ethereum/types/ChannelsMock.d.ts:1400

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

packages/ethereum/types/ChannelsMock.d.ts:523

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

packages/ethereum/types/ChannelsMock.d.ts:525

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

packages/ethereum/types/ChannelsMock.d.ts:527

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

packages/ethereum/types/ChannelsMock.d.ts:529

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

packages/ethereum/types/ChannelsMock.d.ts:533

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

packages/ethereum/types/ChannelsMock.d.ts:538

___

### attach

▸ **attach**(`addressOrName`): [`ChannelsMock`](ChannelsMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[`ChannelsMock`](ChannelsMock.md)

#### Overrides

Contract.attach

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:224

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

packages/ethereum/types/ChannelsMock.d.ts:543

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

packages/ethereum/types/ChannelsMock.d.ts:549

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

packages/ethereum/types/ChannelsMock.d.ts:555

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

packages/ethereum/types/ChannelsMock.d.ts:561

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

packages/ethereum/types/ChannelsMock.d.ts:567

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

packages/ethereum/types/ChannelsMock.d.ts:582

___

### computeChallengeInternal

▸ **computeChallengeInternal**(`response`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `response` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:597

___

### computeChallengeInternal(bytes32)

▸ **computeChallengeInternal(bytes32)**(`response`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `response` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:602

___

### connect

▸ **connect**(`signerOrProvider`): [`ChannelsMock`](ChannelsMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[`ChannelsMock`](ChannelsMock.md)

#### Overrides

Contract.connect

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:223

___

### deployed

▸ **deployed**(): `Promise`<[`ChannelsMock`](ChannelsMock.md)\>

#### Returns

`Promise`<[`ChannelsMock`](ChannelsMock.md)\>

#### Overrides

Contract.deployed

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:225

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

packages/ethereum/types/ChannelsMock.d.ts:607

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

packages/ethereum/types/ChannelsMock.d.ts:612

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

packages/ethereum/types/ChannelsMock.d.ts:617

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

packages/ethereum/types/ChannelsMock.d.ts:625

___

### getChannelIdInternal

▸ **getChannelIdInternal**(`partyA`, `partyB`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `partyA` | `string` |
| `partyB` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:633

___

### getChannelIdInternal(address,address)

▸ **getChannelIdInternal(address,address)**(`partyA`, `partyB`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `partyA` | `string` |
| `partyB` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:639

___

### getEncodedTicketInternal

▸ **getEncodedTicketInternal**(`recipient`, `recipientCounter`, `proofOfRelaySecret`, `channelIteration`, `amount`, `ticketIndex`, `winProb`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `recipient` | `string` |
| `recipientCounter` | `BigNumberish` |
| `proofOfRelaySecret` | `BytesLike` |
| `channelIteration` | `BigNumberish` |
| `amount` | `BigNumberish` |
| `ticketIndex` | `BigNumberish` |
| `winProb` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:645

___

### getEncodedTicketInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)

▸ **getEncodedTicketInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)**(`recipient`, `recipientCounter`, `proofOfRelaySecret`, `channelIteration`, `amount`, `ticketIndex`, `winProb`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `recipient` | `string` |
| `recipientCounter` | `BigNumberish` |
| `proofOfRelaySecret` | `BytesLike` |
| `channelIteration` | `BigNumberish` |
| `amount` | `BigNumberish` |
| `ticketIndex` | `BigNumberish` |
| `winProb` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:656

___

### getTicketHashInternal

▸ **getTicketHashInternal**(`recipient`, `recipientCounter`, `proofOfRelaySecret`, `channelIteration`, `amount`, `ticketIndex`, `winProb`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `recipient` | `string` |
| `recipientCounter` | `BigNumberish` |
| `proofOfRelaySecret` | `BytesLike` |
| `channelIteration` | `BigNumberish` |
| `amount` | `BigNumberish` |
| `ticketIndex` | `BigNumberish` |
| `winProb` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:667

___

### getTicketHashInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)

▸ **getTicketHashInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)**(`recipient`, `recipientCounter`, `proofOfRelaySecret`, `channelIteration`, `amount`, `ticketIndex`, `winProb`, `overrides?`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `recipient` | `string` |
| `recipientCounter` | `BigNumberish` |
| `proofOfRelaySecret` | `BytesLike` |
| `channelIteration` | `BigNumberish` |
| `amount` | `BigNumberish` |
| `ticketIndex` | `BigNumberish` |
| `winProb` | `BigNumberish` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`string`\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:678

___

### getTicketLuckInternal

▸ **getTicketLuckInternal**(`ticketHash`, `secretPreImage`, `proofOfRelaySecret`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticketHash` | `BytesLike` |
| `secretPreImage` | `BytesLike` |
| `proofOfRelaySecret` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:689

___

### getTicketLuckInternal(bytes32,bytes32,bytes32)

▸ **getTicketLuckInternal(bytes32,bytes32,bytes32)**(`ticketHash`, `secretPreImage`, `proofOfRelaySecret`, `overrides?`): `Promise`<`BigNumber`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticketHash` | `BytesLike` |
| `secretPreImage` | `BytesLike` |
| `proofOfRelaySecret` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<`BigNumber`\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:696

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

packages/ethereum/types/ChannelsMock.d.ts:703

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

packages/ethereum/types/ChannelsMock.d.ts:708

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

packages/ethereum/types/ChannelsMock.d.ts:227

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

packages/ethereum/types/ChannelsMock.d.ts:250

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

packages/ethereum/types/ChannelsMock.d.ts:713

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

packages/ethereum/types/ChannelsMock.d.ts:718

___

### off

▸ **off**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ChannelsMock`](ChannelsMock.md)

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

[`ChannelsMock`](ChannelsMock.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:230

▸ **off**(`eventName`, `listener`): [`ChannelsMock`](ChannelsMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ChannelsMock`](ChannelsMock.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:251

___

### on

▸ **on**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ChannelsMock`](ChannelsMock.md)

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

[`ChannelsMock`](ChannelsMock.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:234

▸ **on**(`eventName`, `listener`): [`ChannelsMock`](ChannelsMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ChannelsMock`](ChannelsMock.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:252

___

### once

▸ **once**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ChannelsMock`](ChannelsMock.md)

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

[`ChannelsMock`](ChannelsMock.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:238

▸ **once**(`eventName`, `listener`): [`ChannelsMock`](ChannelsMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ChannelsMock`](ChannelsMock.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:253

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

packages/ethereum/types/ChannelsMock.d.ts:257

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

packages/ethereum/types/ChannelsMock.d.ts:723

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

packages/ethereum/types/ChannelsMock.d.ts:735

___

### removeAllListeners

▸ **removeAllListeners**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`): [`ChannelsMock`](ChannelsMock.md)

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

[`ChannelsMock`](ChannelsMock.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:246

▸ **removeAllListeners**(`eventName?`): [`ChannelsMock`](ChannelsMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[`ChannelsMock`](ChannelsMock.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:255

___

### removeListener

▸ **removeListener**<`EventArgsArray`, `EventArgsObject`\>(`eventFilter`, `listener`): [`ChannelsMock`](ChannelsMock.md)

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

[`ChannelsMock`](ChannelsMock.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:242

▸ **removeListener**(`eventName`, `listener`): [`ChannelsMock`](ChannelsMock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[`ChannelsMock`](ChannelsMock.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:254

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

packages/ethereum/types/ChannelsMock.d.ts:747

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

packages/ethereum/types/ChannelsMock.d.ts:749

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

packages/ethereum/types/ChannelsMock.d.ts:751

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

packages/ethereum/types/ChannelsMock.d.ts:753

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

packages/ethereum/types/ChannelsMock.d.ts:755

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

packages/ethereum/types/ChannelsMock.d.ts:765

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
