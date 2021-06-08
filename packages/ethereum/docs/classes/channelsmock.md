[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / ChannelsMock

# Class: ChannelsMock

## Hierarchy

- `Contract`

  ↳ **ChannelsMock**

## Table of contents

### Constructors

- [constructor](channelsmock.md#constructor)

### Properties

- [\_deployedPromise](channelsmock.md#_deployedpromise)
- [\_runningEvents](channelsmock.md#_runningevents)
- [\_wrappedEmits](channelsmock.md#_wrappedemits)
- [address](channelsmock.md#address)
- [callStatic](channelsmock.md#callstatic)
- [deployTransaction](channelsmock.md#deploytransaction)
- [estimateGas](channelsmock.md#estimategas)
- [filters](channelsmock.md#filters)
- [functions](channelsmock.md#functions)
- [interface](channelsmock.md#interface)
- [populateTransaction](channelsmock.md#populatetransaction)
- [provider](channelsmock.md#provider)
- [resolvedAddress](channelsmock.md#resolvedaddress)
- [signer](channelsmock.md#signer)

### Methods

- [FUND\_CHANNEL\_MULTI\_SIZE](channelsmock.md#fund_channel_multi_size)
- [FUND\_CHANNEL\_MULTI\_SIZE()](channelsmock.md#fund_channel_multi_size())
- [TOKENS\_RECIPIENT\_INTERFACE\_HASH](channelsmock.md#tokens_recipient_interface_hash)
- [TOKENS\_RECIPIENT\_INTERFACE\_HASH()](channelsmock.md#tokens_recipient_interface_hash())
- [\_checkRunningEvents](channelsmock.md#_checkrunningevents)
- [\_deployed](channelsmock.md#_deployed)
- [\_wrapEvent](channelsmock.md#_wrapevent)
- [announce](channelsmock.md#announce)
- [announce(bytes)](channelsmock.md#announce(bytes))
- [attach](channelsmock.md#attach)
- [bumpChannel](channelsmock.md#bumpchannel)
- [bumpChannel(address,bytes32)](channelsmock.md#bumpchannel(address,bytes32))
- [canImplementInterfaceForAddress](channelsmock.md#canimplementinterfaceforaddress)
- [canImplementInterfaceForAddress(bytes32,address)](channelsmock.md#canimplementinterfaceforaddress(bytes32,address))
- [channels](channelsmock.md#channels)
- [channels(bytes32)](channelsmock.md#channels(bytes32))
- [computeChallengeInternal](channelsmock.md#computechallengeinternal)
- [computeChallengeInternal(bytes32)](channelsmock.md#computechallengeinternal(bytes32))
- [connect](channelsmock.md#connect)
- [deployed](channelsmock.md#deployed)
- [emit](channelsmock.md#emit)
- [fallback](channelsmock.md#fallback)
- [finalizeChannelClosure](channelsmock.md#finalizechannelclosure)
- [finalizeChannelClosure(address)](channelsmock.md#finalizechannelclosure(address))
- [fundChannelMulti](channelsmock.md#fundchannelmulti)
- [fundChannelMulti(address,address,uint256,uint256)](channelsmock.md#fundchannelmulti(address,address,uint256,uint256))
- [getChannelIdInternal](channelsmock.md#getchannelidinternal)
- [getChannelIdInternal(address,address)](channelsmock.md#getchannelidinternal(address,address))
- [getChannelInternal](channelsmock.md#getchannelinternal)
- [getChannelInternal(address,address)](channelsmock.md#getchannelinternal(address,address))
- [getEncodedTicketInternal](channelsmock.md#getencodedticketinternal)
- [getEncodedTicketInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)](channelsmock.md#getencodedticketinternal(address,uint256,bytes32,uint256,uint256,uint256,uint256))
- [getPartiesInternal](channelsmock.md#getpartiesinternal)
- [getPartiesInternal(address,address)](channelsmock.md#getpartiesinternal(address,address))
- [getTicketHashInternal](channelsmock.md#gettickethashinternal)
- [getTicketHashInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)](channelsmock.md#gettickethashinternal(address,uint256,bytes32,uint256,uint256,uint256,uint256))
- [getTicketLuckInternal](channelsmock.md#getticketluckinternal)
- [getTicketLuckInternal(bytes32,bytes32,bytes32)](channelsmock.md#getticketluckinternal(bytes32,bytes32,bytes32))
- [initiateChannelClosure](channelsmock.md#initiatechannelclosure)
- [initiateChannelClosure(address)](channelsmock.md#initiatechannelclosure(address))
- [isPartyAInternal](channelsmock.md#ispartyainternal)
- [isPartyAInternal(address,address)](channelsmock.md#ispartyainternal(address,address))
- [listenerCount](channelsmock.md#listenercount)
- [listeners](channelsmock.md#listeners)
- [off](channelsmock.md#off)
- [on](channelsmock.md#on)
- [once](channelsmock.md#once)
- [queryFilter](channelsmock.md#queryfilter)
- [redeemTicket](channelsmock.md#redeemticket)
- [redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)](channelsmock.md#redeemticket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes))
- [removeAllListeners](channelsmock.md#removealllisteners)
- [removeListener](channelsmock.md#removelistener)
- [secsClosure](channelsmock.md#secsclosure)
- [secsClosure()](channelsmock.md#secsclosure())
- [token](channelsmock.md#token)
- [token()](channelsmock.md#token())
- [tokensReceived](channelsmock.md#tokensreceived)
- [tokensReceived(address,address,address,uint256,bytes,bytes)](channelsmock.md#tokensreceived(address,address,address,uint256,bytes,bytes))
- [getContractAddress](channelsmock.md#getcontractaddress)
- [getInterface](channelsmock.md#getinterface)
- [isIndexed](channelsmock.md#isindexed)

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

node_modules/@ethersproject/contracts/lib/index.d.ts:98

## Properties

### \_deployedPromise

• **\_deployedPromise**: `Promise`<Contract\>

#### Inherited from

Contract.\_deployedPromise

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:92

___

### \_runningEvents

• **\_runningEvents**: `Object`

#### Index signature

▪ [eventTag: `string`]: `RunningEvent`

#### Inherited from

Contract.\_runningEvents

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:93

___

### \_wrappedEmits

• **\_wrappedEmits**: `Object`

#### Index signature

▪ [eventTag: `string`]: (...`args`: `any`[]) => `void`

#### Inherited from

Contract.\_wrappedEmits

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:96

___

### address

• `Readonly` **address**: `string`

#### Inherited from

Contract.address

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:71

___

### callStatic

• **callStatic**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `FUND_CHANNEL_MULTI_SIZE` | (`overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `FUND_CHANNEL_MULTI_SIZE()` | (`overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: `CallOverrides`) => `Promise`<string\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: `CallOverrides`) => `Promise`<string\> |
| `announce` | (`multiaddr`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<void\> |
| `announce(bytes)` | (`multiaddr`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<void\> |
| `bumpChannel` | (`counterparty`: `string`, `newCommitment`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<void\> |
| `bumpChannel(address,bytes32)` | (`counterparty`: `string`, `newCommitment`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<void\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<string\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<string\> |
| `channels` | (`arg0`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `string`, `string`, `BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`, `boolean`] & { `channelEpoch`: `BigNumber` ; `closureByPartyA`: `boolean` ; `closureTime`: `number` ; `partyABalance`: `BigNumber` ; `partyACommitment`: `string` ; `partyATicketEpoch`: `BigNumber` ; `partyATicketIndex`: `BigNumber` ; `partyBBalance`: `BigNumber` ; `partyBCommitment`: `string` ; `partyBTicketEpoch`: `BigNumber` ; `partyBTicketIndex`: `BigNumber` ; `status`: `number`  }\> |
| `channels(bytes32)` | (`arg0`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `string`, `string`, `BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`, `boolean`] & { `channelEpoch`: `BigNumber` ; `closureByPartyA`: `boolean` ; `closureTime`: `number` ; `partyABalance`: `BigNumber` ; `partyACommitment`: `string` ; `partyATicketEpoch`: `BigNumber` ; `partyATicketIndex`: `BigNumber` ; `partyBBalance`: `BigNumber` ; `partyBCommitment`: `string` ; `partyBTicketEpoch`: `BigNumber` ; `partyBTicketIndex`: `BigNumber` ; `status`: `number`  }\> |
| `computeChallengeInternal` | (`response`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<string\> |
| `computeChallengeInternal(bytes32)` | (`response`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<string\> |
| `finalizeChannelClosure` | (`counterparty`: `string`, `overrides?`: `CallOverrides`) => `Promise`<void\> |
| `finalizeChannelClosure(address)` | (`counterparty`: `string`, `overrides?`: `CallOverrides`) => `Promise`<void\> |
| `fundChannelMulti` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<void\> |
| `fundChannelMulti(address,address,uint256,uint256)` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<void\> |
| `getChannelIdInternal` | (`partyA`: `string`, `partyB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<string\> |
| `getChannelIdInternal(address,address)` | (`partyA`: `string`, `partyB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<string\> |
| `getChannelInternal` | (`accountA`: `string`, `accountB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`, `string`, `string`]\> |
| `getChannelInternal(address,address)` | (`accountA`: `string`, `accountB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`, `string`, `string`]\> |
| `getEncodedTicketInternal` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<string\> |
| `getEncodedTicketInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<string\> |
| `getPartiesInternal` | (`account1`: `string`, `account2`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`, `string`]\> |
| `getPartiesInternal(address,address)` | (`account1`: `string`, `account2`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`, `string`]\> |
| `getTicketHashInternal` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<string\> |
| `getTicketHashInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<string\> |
| `getTicketLuckInternal` | (`ticketHash`: `BytesLike`, `secretPreImage`: `BytesLike`, `proofOfRelaySecret`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `getTicketLuckInternal(bytes32,bytes32,bytes32)` | (`ticketHash`: `BytesLike`, `secretPreImage`: `BytesLike`, `proofOfRelaySecret`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `initiateChannelClosure` | (`counterparty`: `string`, `overrides?`: `CallOverrides`) => `Promise`<void\> |
| `initiateChannelClosure(address)` | (`counterparty`: `string`, `overrides?`: `CallOverrides`) => `Promise`<void\> |
| `isPartyAInternal` | (`accountA`: `string`, `accountB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<boolean\> |
| `isPartyAInternal(address,address)` | (`accountA`: `string`, `accountB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<boolean\> |
| `redeemTicket` | (`counterparty`: `string`, `nextCommitment`: `BytesLike`, `ticketEpoch`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `amount`: `BigNumberish`, `winProb`: `BigNumberish`, `signature`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<void\> |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | (`counterparty`: `string`, `nextCommitment`: `BytesLike`, `ticketEpoch`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `amount`: `BigNumberish`, `winProb`: `BigNumberish`, `signature`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<void\> |
| `secsClosure` | (`overrides?`: `CallOverrides`) => `Promise`<number\> |
| `secsClosure()` | (`overrides?`: `CallOverrides`) => `Promise`<number\> |
| `token` | (`overrides?`: `CallOverrides`) => `Promise`<string\> |
| `token()` | (`overrides?`: `CallOverrides`) => `Promise`<string\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<void\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<void\> |

#### Overrides

Contract.callStatic

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:910

___

### deployTransaction

• `Readonly` **deployTransaction**: `TransactionResponse`

#### Inherited from

Contract.deployTransaction

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:91

___

### estimateGas

• **estimateGas**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `FUND_CHANNEL_MULTI_SIZE` | (`overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `FUND_CHANNEL_MULTI_SIZE()` | (`overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `announce` | (`multiaddr`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<BigNumber\> |
| `announce(bytes)` | (`multiaddr`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<BigNumber\> |
| `bumpChannel` | (`counterparty`: `string`, `newCommitment`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<BigNumber\> |
| `bumpChannel(address,bytes32)` | (`counterparty`: `string`, `newCommitment`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<BigNumber\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `channels` | (`arg0`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `channels(bytes32)` | (`arg0`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `computeChallengeInternal` | (`response`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `computeChallengeInternal(bytes32)` | (`response`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `finalizeChannelClosure` | (`counterparty`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<BigNumber\> |
| `finalizeChannelClosure(address)` | (`counterparty`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<BigNumber\> |
| `fundChannelMulti` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<BigNumber\> |
| `fundChannelMulti(address,address,uint256,uint256)` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<BigNumber\> |
| `getChannelIdInternal` | (`partyA`: `string`, `partyB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `getChannelIdInternal(address,address)` | (`partyA`: `string`, `partyB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `getChannelInternal` | (`accountA`: `string`, `accountB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `getChannelInternal(address,address)` | (`accountA`: `string`, `accountB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `getEncodedTicketInternal` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `getEncodedTicketInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `getPartiesInternal` | (`account1`: `string`, `account2`: `string`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `getPartiesInternal(address,address)` | (`account1`: `string`, `account2`: `string`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `getTicketHashInternal` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `getTicketHashInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `getTicketLuckInternal` | (`ticketHash`: `BytesLike`, `secretPreImage`: `BytesLike`, `proofOfRelaySecret`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `getTicketLuckInternal(bytes32,bytes32,bytes32)` | (`ticketHash`: `BytesLike`, `secretPreImage`: `BytesLike`, `proofOfRelaySecret`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `initiateChannelClosure` | (`counterparty`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<BigNumber\> |
| `initiateChannelClosure(address)` | (`counterparty`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<BigNumber\> |
| `isPartyAInternal` | (`accountA`: `string`, `accountB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `isPartyAInternal(address,address)` | (`accountA`: `string`, `accountB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `redeemTicket` | (`counterparty`: `string`, `nextCommitment`: `BytesLike`, `ticketEpoch`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `amount`: `BigNumberish`, `winProb`: `BigNumberish`, `signature`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<BigNumber\> |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | (`counterparty`: `string`, `nextCommitment`: `BytesLike`, `ticketEpoch`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `amount`: `BigNumberish`, `winProb`: `BigNumberish`, `signature`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<BigNumber\> |
| `secsClosure` | (`overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `secsClosure()` | (`overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `token` | (`overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `token()` | (`overrides?`: `CallOverrides`) => `Promise`<BigNumber\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<BigNumber\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<BigNumber\> |

#### Overrides

Contract.estimateGas

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:1302

___

### filters

• **filters**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Announcement` | (`account`: `string`, `multiaddr`: ``null``) => [TypedEventFilter](../interfaces/typedeventfilter.md)<[`string`, `string`], `Object`\> |
| `ChannelUpdate` | (`partyA`: `string`, `partyB`: `string`, `newState`: ``null``) => [TypedEventFilter](../interfaces/typedeventfilter.md)<[`string`, `string`, [`BigNumber`, `BigNumber`, `string`, `string`, `BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`, `boolean`] & { `channelEpoch`: `BigNumber` ; `closureByPartyA`: `boolean` ; `closureTime`: `number` ; `partyABalance`: `BigNumber` ; `partyACommitment`: `string` ; `partyATicketEpoch`: `BigNumber` ; `partyATicketIndex`: `BigNumber` ; `partyBBalance`: `BigNumber` ; `partyBCommitment`: `string` ; `partyBTicketEpoch`: `BigNumber` ; `partyBTicketIndex`: `BigNumber` ; `status`: `number`  }], `Object`\> |

#### Overrides

Contract.filters

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:1223

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
| `announce` | (`multiaddr`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<ContractTransaction\> |
| `announce(bytes)` | (`multiaddr`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<ContractTransaction\> |
| `bumpChannel` | (`counterparty`: `string`, `newCommitment`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<ContractTransaction\> |
| `bumpChannel(address,bytes32)` | (`counterparty`: `string`, `newCommitment`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<ContractTransaction\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `channels` | (`arg0`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `string`, `string`, `BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`, `boolean`] & { `channelEpoch`: `BigNumber` ; `closureByPartyA`: `boolean` ; `closureTime`: `number` ; `partyABalance`: `BigNumber` ; `partyACommitment`: `string` ; `partyATicketEpoch`: `BigNumber` ; `partyATicketIndex`: `BigNumber` ; `partyBBalance`: `BigNumber` ; `partyBCommitment`: `string` ; `partyBTicketEpoch`: `BigNumber` ; `partyBTicketIndex`: `BigNumber` ; `status`: `number`  }\> |
| `channels(bytes32)` | (`arg0`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`, `BigNumber`, `string`, `string`, `BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`, `boolean`] & { `channelEpoch`: `BigNumber` ; `closureByPartyA`: `boolean` ; `closureTime`: `number` ; `partyABalance`: `BigNumber` ; `partyACommitment`: `string` ; `partyATicketEpoch`: `BigNumber` ; `partyATicketIndex`: `BigNumber` ; `partyBBalance`: `BigNumber` ; `partyBCommitment`: `string` ; `partyBTicketEpoch`: `BigNumber` ; `partyBTicketIndex`: `BigNumber` ; `status`: `number`  }\> |
| `computeChallengeInternal` | (`response`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `computeChallengeInternal(bytes32)` | (`response`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `finalizeChannelClosure` | (`counterparty`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<ContractTransaction\> |
| `finalizeChannelClosure(address)` | (`counterparty`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<ContractTransaction\> |
| `fundChannelMulti` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<ContractTransaction\> |
| `fundChannelMulti(address,address,uint256,uint256)` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<ContractTransaction\> |
| `getChannelIdInternal` | (`partyA`: `string`, `partyB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getChannelIdInternal(address,address)` | (`partyA`: `string`, `partyB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getChannelInternal` | (`accountA`: `string`, `accountB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`, `string`, `string`]\> |
| `getChannelInternal(address,address)` | (`accountA`: `string`, `accountB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`, `string`, `string`]\> |
| `getEncodedTicketInternal` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getEncodedTicketInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getPartiesInternal` | (`account1`: `string`, `account2`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`, `string`]\> |
| `getPartiesInternal(address,address)` | (`account1`: `string`, `account2`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`string`, `string`]\> |
| `getTicketHashInternal` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getTicketHashInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `getTicketLuckInternal` | (`ticketHash`: `BytesLike`, `secretPreImage`: `BytesLike`, `proofOfRelaySecret`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `getTicketLuckInternal(bytes32,bytes32,bytes32)` | (`ticketHash`: `BytesLike`, `secretPreImage`: `BytesLike`, `proofOfRelaySecret`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<[`BigNumber`]\> |
| `initiateChannelClosure` | (`counterparty`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<ContractTransaction\> |
| `initiateChannelClosure(address)` | (`counterparty`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<ContractTransaction\> |
| `isPartyAInternal` | (`accountA`: `string`, `accountB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `isPartyAInternal(address,address)` | (`accountA`: `string`, `accountB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<[`boolean`]\> |
| `redeemTicket` | (`counterparty`: `string`, `nextCommitment`: `BytesLike`, `ticketEpoch`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `amount`: `BigNumberish`, `winProb`: `BigNumberish`, `signature`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<ContractTransaction\> |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | (`counterparty`: `string`, `nextCommitment`: `BytesLike`, `ticketEpoch`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `amount`: `BigNumberish`, `winProb`: `BigNumberish`, `signature`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<ContractTransaction\> |
| `secsClosure` | (`overrides?`: `CallOverrides`) => `Promise`<[`number`]\> |
| `secsClosure()` | (`overrides?`: `CallOverrides`) => `Promise`<[`number`]\> |
| `token` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `token()` | (`overrides?`: `CallOverrides`) => `Promise`<[`string`]\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<ContractTransaction\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<ContractTransaction\> |

#### Overrides

Contract.functions

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:276

___

### interface

• **interface**: `ChannelsMockInterface`

#### Overrides

Contract.interface

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:274

___

### populateTransaction

• **populateTransaction**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `FUND_CHANNEL_MULTI_SIZE` | (`overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `FUND_CHANNEL_MULTI_SIZE()` | (`overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `announce` | (`multiaddr`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<PopulatedTransaction\> |
| `announce(bytes)` | (`multiaddr`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<PopulatedTransaction\> |
| `bumpChannel` | (`counterparty`: `string`, `newCommitment`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<PopulatedTransaction\> |
| `bumpChannel(address,bytes32)` | (`counterparty`: `string`, `newCommitment`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<PopulatedTransaction\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: `BytesLike`, `account`: `string`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `channels` | (`arg0`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `channels(bytes32)` | (`arg0`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `computeChallengeInternal` | (`response`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `computeChallengeInternal(bytes32)` | (`response`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `finalizeChannelClosure` | (`counterparty`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<PopulatedTransaction\> |
| `finalizeChannelClosure(address)` | (`counterparty`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<PopulatedTransaction\> |
| `fundChannelMulti` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<PopulatedTransaction\> |
| `fundChannelMulti(address,address,uint256,uint256)` | (`account1`: `string`, `account2`: `string`, `amount1`: `BigNumberish`, `amount2`: `BigNumberish`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<PopulatedTransaction\> |
| `getChannelIdInternal` | (`partyA`: `string`, `partyB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `getChannelIdInternal(address,address)` | (`partyA`: `string`, `partyB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `getChannelInternal` | (`accountA`: `string`, `accountB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `getChannelInternal(address,address)` | (`accountA`: `string`, `accountB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `getEncodedTicketInternal` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `getEncodedTicketInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `getPartiesInternal` | (`account1`: `string`, `account2`: `string`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `getPartiesInternal(address,address)` | (`account1`: `string`, `account2`: `string`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `getTicketHashInternal` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `getTicketHashInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)` | (`recipient`: `string`, `recipientCounter`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `channelIteration`: `BigNumberish`, `amount`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `winProb`: `BigNumberish`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `getTicketLuckInternal` | (`ticketHash`: `BytesLike`, `secretPreImage`: `BytesLike`, `proofOfRelaySecret`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `getTicketLuckInternal(bytes32,bytes32,bytes32)` | (`ticketHash`: `BytesLike`, `secretPreImage`: `BytesLike`, `proofOfRelaySecret`: `BytesLike`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `initiateChannelClosure` | (`counterparty`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<PopulatedTransaction\> |
| `initiateChannelClosure(address)` | (`counterparty`: `string`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<PopulatedTransaction\> |
| `isPartyAInternal` | (`accountA`: `string`, `accountB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `isPartyAInternal(address,address)` | (`accountA`: `string`, `accountB`: `string`, `overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `redeemTicket` | (`counterparty`: `string`, `nextCommitment`: `BytesLike`, `ticketEpoch`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `amount`: `BigNumberish`, `winProb`: `BigNumberish`, `signature`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<PopulatedTransaction\> |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | (`counterparty`: `string`, `nextCommitment`: `BytesLike`, `ticketEpoch`: `BigNumberish`, `ticketIndex`: `BigNumberish`, `proofOfRelaySecret`: `BytesLike`, `amount`: `BigNumberish`, `winProb`: `BigNumberish`, `signature`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<PopulatedTransaction\> |
| `secsClosure` | (`overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `secsClosure()` | (`overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `token` | (`overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `token()` | (`overrides?`: `CallOverrides`) => `Promise`<PopulatedTransaction\> |
| `tokensReceived` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<PopulatedTransaction\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: `string`, `from`: `string`, `to`: `string`, `amount`: `BigNumberish`, `userData`: `BytesLike`, `operatorData`: `BytesLike`, `overrides?`: `Overrides` & { `from?`: `string` \| `Promise`<string\>  }) => `Promise`<PopulatedTransaction\> |

#### Overrides

Contract.populateTransaction

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:1561

___

### provider

• `Readonly` **provider**: `Provider`

#### Inherited from

Contract.provider

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:74

___

### resolvedAddress

• `Readonly` **resolvedAddress**: `Promise`<string\>

#### Inherited from

Contract.resolvedAddress

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:90

___

### signer

• `Readonly` **signer**: `Signer`

#### Inherited from

Contract.signer

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:73

## Methods

### FUND\_CHANNEL\_MULTI\_SIZE

▸ **FUND_CHANNEL_MULTI_SIZE**(`overrides?`): `Promise`<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<BigNumber\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:596

___

### FUND\_CHANNEL\_MULTI\_SIZE()

▸ **FUND_CHANNEL_MULTI_SIZE()**(`overrides?`): `Promise`<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<BigNumber\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:596

___

### TOKENS\_RECIPIENT\_INTERFACE\_HASH

▸ **TOKENS_RECIPIENT_INTERFACE_HASH**(`overrides?`): `Promise`<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<string\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:600

___

### TOKENS\_RECIPIENT\_INTERFACE\_HASH()

▸ **TOKENS_RECIPIENT_INTERFACE_HASH()**(`overrides?`): `Promise`<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<string\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:600

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

node_modules/@ethersproject/contracts/lib/index.d.ts:113

___

### \_deployed

▸ **_deployed**(`blockTag?`): `Promise`<Contract\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockTag?` | `BlockTag` |

#### Returns

`Promise`<Contract\>

#### Inherited from

Contract.\_deployed

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:106

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

node_modules/@ethersproject/contracts/lib/index.d.ts:114

___

### announce

▸ **announce**(`multiaddr`, `overrides?`): `Promise`<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `multiaddr` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<string\>  } |

#### Returns

`Promise`<ContractTransaction\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:606

___

### announce(bytes)

▸ **announce(bytes)**(`multiaddr`, `overrides?`): `Promise`<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `multiaddr` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<string\>  } |

#### Returns

`Promise`<ContractTransaction\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:609

___

### attach

▸ **attach**(`addressOrName`): [ChannelsMock](channelsmock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | `string` |

#### Returns

[ChannelsMock](channelsmock.md)

#### Overrides

Contract.attach

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:235

___

### bumpChannel

▸ **bumpChannel**(`counterparty`, `newCommitment`, `overrides?`): `Promise`<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | `string` |
| `newCommitment` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<string\>  } |

#### Returns

`Promise`<ContractTransaction\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:616

___

### bumpChannel(address,bytes32)

▸ **bumpChannel(address,bytes32)**(`counterparty`, `newCommitment`, `overrides?`): `Promise`<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | `string` |
| `newCommitment` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<string\>  } |

#### Returns

`Promise`<ContractTransaction\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:620

___

### canImplementInterfaceForAddress

▸ **canImplementInterfaceForAddress**(`interfaceHash`, `account`, `overrides?`): `Promise`<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `interfaceHash` | `BytesLike` |
| `account` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<string\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:628

___

### canImplementInterfaceForAddress(bytes32,address)

▸ **canImplementInterfaceForAddress(bytes32,address)**(`interfaceHash`, `account`, `overrides?`): `Promise`<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `interfaceHash` | `BytesLike` |
| `account` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<string\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:632

___

### channels

▸ **channels**(`arg0`, `overrides?`): `Promise`<[`BigNumber`, `BigNumber`, `string`, `string`, `BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`, `boolean`] & { `channelEpoch`: `BigNumber` ; `closureByPartyA`: `boolean` ; `closureTime`: `number` ; `partyABalance`: `BigNumber` ; `partyACommitment`: `string` ; `partyATicketEpoch`: `BigNumber` ; `partyATicketIndex`: `BigNumber` ; `partyBBalance`: `BigNumber` ; `partyBCommitment`: `string` ; `partyBTicketEpoch`: `BigNumber` ; `partyBTicketIndex`: `BigNumber` ; `status`: `number`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`BigNumber`, `BigNumber`, `string`, `string`, `BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`, `boolean`] & { `channelEpoch`: `BigNumber` ; `closureByPartyA`: `boolean` ; `closureTime`: `number` ; `partyABalance`: `BigNumber` ; `partyACommitment`: `string` ; `partyATicketEpoch`: `BigNumber` ; `partyATicketIndex`: `BigNumber` ; `partyBBalance`: `BigNumber` ; `partyBCommitment`: `string` ; `partyBTicketEpoch`: `BigNumber` ; `partyBTicketIndex`: `BigNumber` ; `status`: `number`  }\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:640

___

### channels(bytes32)

▸ **channels(bytes32)**(`arg0`, `overrides?`): `Promise`<[`BigNumber`, `BigNumber`, `string`, `string`, `BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`, `boolean`] & { `channelEpoch`: `BigNumber` ; `closureByPartyA`: `boolean` ; `closureTime`: `number` ; `partyABalance`: `BigNumber` ; `partyACommitment`: `string` ; `partyATicketEpoch`: `BigNumber` ; `partyATicketIndex`: `BigNumber` ; `partyBBalance`: `BigNumber` ; `partyBCommitment`: `string` ; `partyBTicketEpoch`: `BigNumber` ; `partyBTicketIndex`: `BigNumber` ; `status`: `number`  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`BigNumber`, `BigNumber`, `string`, `string`, `BigNumber`, `BigNumber`, `BigNumber`, `BigNumber`, `number`, `BigNumber`, `number`, `boolean`] & { `channelEpoch`: `BigNumber` ; `closureByPartyA`: `boolean` ; `closureTime`: `number` ; `partyABalance`: `BigNumber` ; `partyACommitment`: `string` ; `partyATicketEpoch`: `BigNumber` ; `partyATicketIndex`: `BigNumber` ; `partyBBalance`: `BigNumber` ; `partyBCommitment`: `string` ; `partyBTicketEpoch`: `BigNumber` ; `partyBTicketIndex`: `BigNumber` ; `status`: `number`  }\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:671

___

### computeChallengeInternal

▸ **computeChallengeInternal**(`response`, `overrides?`): `Promise`<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `response` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<string\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:706

___

### computeChallengeInternal(bytes32)

▸ **computeChallengeInternal(bytes32)**(`response`, `overrides?`): `Promise`<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `response` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<string\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:709

___

### connect

▸ **connect**(`signerOrProvider`): [ChannelsMock](channelsmock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | `string` \| `Signer` \| `Provider` |

#### Returns

[ChannelsMock](channelsmock.md)

#### Overrides

Contract.connect

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:234

___

### deployed

▸ **deployed**(): `Promise`<[ChannelsMock](channelsmock.md)\>

#### Returns

`Promise`<[ChannelsMock](channelsmock.md)\>

#### Overrides

Contract.deployed

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:236

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

node_modules/@ethersproject/contracts/lib/index.d.ts:119

___

### fallback

▸ **fallback**(`overrides?`): `Promise`<TransactionResponse\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `TransactionRequest` |

#### Returns

`Promise`<TransactionResponse\>

#### Inherited from

Contract.fallback

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:107

___

### finalizeChannelClosure

▸ **finalizeChannelClosure**(`counterparty`, `overrides?`): `Promise`<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<string\>  } |

#### Returns

`Promise`<ContractTransaction\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:716

___

### finalizeChannelClosure(address)

▸ **finalizeChannelClosure(address)**(`counterparty`, `overrides?`): `Promise`<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<string\>  } |

#### Returns

`Promise`<ContractTransaction\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:719

___

### fundChannelMulti

▸ **fundChannelMulti**(`account1`, `account2`, `amount1`, `amount2`, `overrides?`): `Promise`<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account1` | `string` |
| `account2` | `string` |
| `amount1` | `BigNumberish` |
| `amount2` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<string\>  } |

#### Returns

`Promise`<ContractTransaction\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:726

___

### fundChannelMulti(address,address,uint256,uint256)

▸ **fundChannelMulti(address,address,uint256,uint256)**(`account1`, `account2`, `amount1`, `amount2`, `overrides?`): `Promise`<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account1` | `string` |
| `account2` | `string` |
| `amount1` | `BigNumberish` |
| `amount2` | `BigNumberish` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<string\>  } |

#### Returns

`Promise`<ContractTransaction\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:732

___

### getChannelIdInternal

▸ **getChannelIdInternal**(`partyA`, `partyB`, `overrides?`): `Promise`<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `partyA` | `string` |
| `partyB` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<string\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:742

___

### getChannelIdInternal(address,address)

▸ **getChannelIdInternal(address,address)**(`partyA`, `partyB`, `overrides?`): `Promise`<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `partyA` | `string` |
| `partyB` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<string\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:746

___

### getChannelInternal

▸ **getChannelInternal**(`accountA`, `accountB`, `overrides?`): `Promise`<[`string`, `string`, `string`]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `accountA` | `string` |
| `accountB` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`string`, `string`, `string`]\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:754

___

### getChannelInternal(address,address)

▸ **getChannelInternal(address,address)**(`accountA`, `accountB`, `overrides?`): `Promise`<[`string`, `string`, `string`]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `accountA` | `string` |
| `accountB` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`string`, `string`, `string`]\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:758

___

### getEncodedTicketInternal

▸ **getEncodedTicketInternal**(`recipient`, `recipientCounter`, `proofOfRelaySecret`, `channelIteration`, `amount`, `ticketIndex`, `winProb`, `overrides?`): `Promise`<string\>

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

`Promise`<string\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:766

___

### getEncodedTicketInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)

▸ **getEncodedTicketInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)**(`recipient`, `recipientCounter`, `proofOfRelaySecret`, `channelIteration`, `amount`, `ticketIndex`, `winProb`, `overrides?`): `Promise`<string\>

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

`Promise`<string\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:775

___

### getPartiesInternal

▸ **getPartiesInternal**(`account1`, `account2`, `overrides?`): `Promise`<[`string`, `string`]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account1` | `string` |
| `account2` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`string`, `string`]\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:788

___

### getPartiesInternal(address,address)

▸ **getPartiesInternal(address,address)**(`account1`, `account2`, `overrides?`): `Promise`<[`string`, `string`]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account1` | `string` |
| `account2` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<[`string`, `string`]\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:792

___

### getTicketHashInternal

▸ **getTicketHashInternal**(`recipient`, `recipientCounter`, `proofOfRelaySecret`, `channelIteration`, `amount`, `ticketIndex`, `winProb`, `overrides?`): `Promise`<string\>

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

`Promise`<string\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:800

___

### getTicketHashInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)

▸ **getTicketHashInternal(address,uint256,bytes32,uint256,uint256,uint256,uint256)**(`recipient`, `recipientCounter`, `proofOfRelaySecret`, `channelIteration`, `amount`, `ticketIndex`, `winProb`, `overrides?`): `Promise`<string\>

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

`Promise`<string\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:809

___

### getTicketLuckInternal

▸ **getTicketLuckInternal**(`ticketHash`, `secretPreImage`, `proofOfRelaySecret`, `overrides?`): `Promise`<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticketHash` | `BytesLike` |
| `secretPreImage` | `BytesLike` |
| `proofOfRelaySecret` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<BigNumber\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:822

___

### getTicketLuckInternal(bytes32,bytes32,bytes32)

▸ **getTicketLuckInternal(bytes32,bytes32,bytes32)**(`ticketHash`, `secretPreImage`, `proofOfRelaySecret`, `overrides?`): `Promise`<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticketHash` | `BytesLike` |
| `secretPreImage` | `BytesLike` |
| `proofOfRelaySecret` | `BytesLike` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<BigNumber\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:827

___

### initiateChannelClosure

▸ **initiateChannelClosure**(`counterparty`, `overrides?`): `Promise`<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<string\>  } |

#### Returns

`Promise`<ContractTransaction\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:836

___

### initiateChannelClosure(address)

▸ **initiateChannelClosure(address)**(`counterparty`, `overrides?`): `Promise`<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | `string` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<string\>  } |

#### Returns

`Promise`<ContractTransaction\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:839

___

### isPartyAInternal

▸ **isPartyAInternal**(`accountA`, `accountB`, `overrides?`): `Promise`<boolean\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `accountA` | `string` |
| `accountB` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<boolean\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:846

___

### isPartyAInternal(address,address)

▸ **isPartyAInternal(address,address)**(`accountA`, `accountB`, `overrides?`): `Promise`<boolean\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `accountA` | `string` |
| `accountB` | `string` |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<boolean\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:850

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

node_modules/@ethersproject/contracts/lib/index.d.ts:120

___

### listeners

▸ **listeners**<EventArgsArray, EventArgsObject\>(`eventFilter?`): [TypedListener](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>[]

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | `EventArgsArray`: `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter?` | [TypedEventFilter](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

#### Returns

[TypedListener](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\>[]

#### Overrides

Contract.listeners

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:238

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

packages/ethereum/types/ChannelsMock.d.ts:261

___

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`, `listener`): [ChannelsMock](channelsmock.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | `EventArgsArray`: `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [TypedEventFilter](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [TypedListener](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\> |

#### Returns

[ChannelsMock](channelsmock.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:241

▸ **off**(`eventName`, `listener`): [ChannelsMock](channelsmock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[ChannelsMock](channelsmock.md)

#### Overrides

Contract.off

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:262

___

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`, `listener`): [ChannelsMock](channelsmock.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | `EventArgsArray`: `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [TypedEventFilter](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [TypedListener](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\> |

#### Returns

[ChannelsMock](channelsmock.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:245

▸ **on**(`eventName`, `listener`): [ChannelsMock](channelsmock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[ChannelsMock](channelsmock.md)

#### Overrides

Contract.on

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:263

___

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`, `listener`): [ChannelsMock](channelsmock.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | `EventArgsArray`: `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [TypedEventFilter](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [TypedListener](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\> |

#### Returns

[ChannelsMock](channelsmock.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:249

▸ **once**(`eventName`, `listener`): [ChannelsMock](channelsmock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[ChannelsMock](channelsmock.md)

#### Overrides

Contract.once

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:264

___

### queryFilter

▸ **queryFilter**<EventArgsArray, EventArgsObject\>(`event`, `fromBlockOrBlockhash?`, `toBlock?`): `Promise`<[TypedEvent](../interfaces/typedevent.md)<`EventArgsArray` & `EventArgsObject`\>[]\>

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | `EventArgsArray`: `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | [TypedEventFilter](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `fromBlockOrBlockhash?` | `string` \| `number` |
| `toBlock?` | `string` \| `number` |

#### Returns

`Promise`<[TypedEvent](../interfaces/typedevent.md)<`EventArgsArray` & `EventArgsObject`\>[]\>

#### Overrides

Contract.queryFilter

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:268

___

### redeemTicket

▸ **redeemTicket**(`counterparty`, `nextCommitment`, `ticketEpoch`, `ticketIndex`, `proofOfRelaySecret`, `amount`, `winProb`, `signature`, `overrides?`): `Promise`<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | `string` |
| `nextCommitment` | `BytesLike` |
| `ticketEpoch` | `BigNumberish` |
| `ticketIndex` | `BigNumberish` |
| `proofOfRelaySecret` | `BytesLike` |
| `amount` | `BigNumberish` |
| `winProb` | `BigNumberish` |
| `signature` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<string\>  } |

#### Returns

`Promise`<ContractTransaction\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:858

___

### redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)

▸ **redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)**(`counterparty`, `nextCommitment`, `ticketEpoch`, `ticketIndex`, `proofOfRelaySecret`, `amount`, `winProb`, `signature`, `overrides?`): `Promise`<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | `string` |
| `nextCommitment` | `BytesLike` |
| `ticketEpoch` | `BigNumberish` |
| `ticketIndex` | `BigNumberish` |
| `proofOfRelaySecret` | `BytesLike` |
| `amount` | `BigNumberish` |
| `winProb` | `BigNumberish` |
| `signature` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<string\>  } |

#### Returns

`Promise`<ContractTransaction\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:868

___

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`): [ChannelsMock](channelsmock.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | `EventArgsArray`: `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [TypedEventFilter](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

#### Returns

[ChannelsMock](channelsmock.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:257

▸ **removeAllListeners**(`eventName?`): [ChannelsMock](channelsmock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | `string` |

#### Returns

[ChannelsMock](channelsmock.md)

#### Overrides

Contract.removeAllListeners

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:266

___

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`, `listener`): [ChannelsMock](channelsmock.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | `EventArgsArray`: `any`[] |
| `EventArgsObject` | `EventArgsObject` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [TypedEventFilter](../interfaces/typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [TypedListener](../modules.md#typedlistener)<EventArgsArray, EventArgsObject\> |

#### Returns

[ChannelsMock](channelsmock.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:253

▸ **removeListener**(`eventName`, `listener`): [ChannelsMock](channelsmock.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | `string` |
| `listener` | `Listener` |

#### Returns

[ChannelsMock](channelsmock.md)

#### Overrides

Contract.removeListener

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:265

___

### secsClosure

▸ **secsClosure**(`overrides?`): `Promise`<number\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<number\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:882

___

### secsClosure()

▸ **secsClosure()**(`overrides?`): `Promise`<number\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<number\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:882

___

### token

▸ **token**(`overrides?`): `Promise`<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<string\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:886

___

### token()

▸ **token()**(`overrides?`): `Promise`<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | `CallOverrides` |

#### Returns

`Promise`<string\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:886

___

### tokensReceived

▸ **tokensReceived**(`operator`, `from`, `to`, `amount`, `userData`, `operatorData`, `overrides?`): `Promise`<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | `string` |
| `from` | `string` |
| `to` | `string` |
| `amount` | `BigNumberish` |
| `userData` | `BytesLike` |
| `operatorData` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<string\>  } |

#### Returns

`Promise`<ContractTransaction\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:890

___

### tokensReceived(address,address,address,uint256,bytes,bytes)

▸ **tokensReceived(address,address,address,uint256,bytes,bytes)**(`operator`, `from`, `to`, `amount`, `userData`, `operatorData`, `overrides?`): `Promise`<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | `string` |
| `from` | `string` |
| `to` | `string` |
| `amount` | `BigNumberish` |
| `userData` | `BytesLike` |
| `operatorData` | `BytesLike` |
| `overrides?` | `Overrides` & { `from?`: `string` \| `Promise`<string\>  } |

#### Returns

`Promise`<ContractTransaction\>

#### Defined in

packages/ethereum/types/ChannelsMock.d.ts:898

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

node_modules/@ethersproject/contracts/lib/index.d.ts:100

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

node_modules/@ethersproject/contracts/lib/index.d.ts:104

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

node_modules/@ethersproject/contracts/lib/index.d.ts:110
