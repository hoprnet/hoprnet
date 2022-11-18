[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / Indexer

# Class: Indexer

Indexes HoprChannels smart contract and stores to the DB,
all channels in the network.
Also keeps track of the latest block number.

## Hierarchy

- `IndexerEventEmitter`<`this`\>

  ↳ **`Indexer`**

## Table of contents

### Constructors

- [constructor](Indexer.md#constructor)

### Properties

- [address](Indexer.md#address)
- [blockProcessingLock](Indexer.md#blockprocessinglock)
- [blockRange](Indexer.md#blockrange)
- [chain](Indexer.md#chain)
- [db](Indexer.md#db)
- [genesisBlock](Indexer.md#genesisblock)
- [lastSnapshot](Indexer.md#lastsnapshot)
- [latestBlock](Indexer.md#latestblock)
- [maxConfirmations](Indexer.md#maxconfirmations)
- [startupBlock](Indexer.md#startupblock)
- [status](Indexer.md#status)
- [unconfirmedEvents](Indexer.md#unconfirmedevents)
- [unsubscribeBlock](Indexer.md#unsubscribeblock)
- [unsubscribeErrors](Indexer.md#unsubscribeerrors)

### Methods

- [addListener](Indexer.md#addlistener)
- [emit](Indexer.md#emit)
- [getAccount](Indexer.md#getaccount)
- [getAddressesAnnouncedOnChain](Indexer.md#getaddressesannouncedonchain)
- [getEvents](Indexer.md#getevents)
- [getOpenChannelsFrom](Indexer.md#getopenchannelsfrom)
- [getPublicKeyOf](Indexer.md#getpublickeyof)
- [getPublicNodes](Indexer.md#getpublicnodes)
- [getRandomOpenChannel](Indexer.md#getrandomopenchannel)
- [indexEvent](Indexer.md#indexevent)
- [listeners](Indexer.md#listeners)
- [off](Indexer.md#off)
- [on](Indexer.md#on)
- [onAnnouncement](Indexer.md#onannouncement)
- [onChannelClosed](Indexer.md#onchannelclosed)
- [onChannelUpdated](Indexer.md#onchannelupdated)
- [onDeregistered](Indexer.md#onderegistered)
- [onEligibilityUpdated](Indexer.md#oneligibilityupdated)
- [onEnabledNetworkRegistry](Indexer.md#onenablednetworkregistry)
- [onNewBlock](Indexer.md#onnewblock)
- [onNewEvents](Indexer.md#onnewevents)
- [onProviderError](Indexer.md#onprovidererror)
- [onRegistered](Indexer.md#onregistered)
- [onTicketRedeemed](Indexer.md#onticketredeemed)
- [onTransfer](Indexer.md#ontransfer)
- [once](Indexer.md#once)
- [prependListener](Indexer.md#prependlistener)
- [prependOnceListener](Indexer.md#prependoncelistener)
- [processPastEvents](Indexer.md#processpastevents)
- [processUnconfirmedEvents](Indexer.md#processunconfirmedevents)
- [removeListener](Indexer.md#removelistener)
- [resolvePendingTransaction](Indexer.md#resolvependingtransaction)
- [restart](Indexer.md#restart)
- [start](Indexer.md#start)
- [stop](Indexer.md#stop)

## Constructors

### constructor

• **new Indexer**(`address`, `db`, `maxConfirmations`, `blockRange`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `Address` |
| `db` | `HoprDB` |
| `maxConfirmations` | `number` |
| `blockRange` | `number` |

#### Overrides

(EventEmitter as new () &#x3D;\&gt; IndexerEventEmitter).constructor

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:107](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L107)

## Properties

### address

• `Private` **address**: `Address`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:108](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L108)

___

### blockProcessingLock

• `Private` **blockProcessingLock**: `DeferType`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:102](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L102)

___

### blockRange

• `Private` **blockRange**: `number`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:111](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L111)

___

### chain

• `Private` **chain**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `announce` | (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `finalizeChannelClosure` | (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `fundChannel` | (`partyA`: `Address`, `partyB`: `Address`, `fundsA`: `Balance`, `fundsB`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `getAllQueuingTransactionRequests` | () => `TransactionRequest`[] |
| `getAllUnconfirmedHash` | () => `string`[] |
| `getBalance` | (`accountAddress`: `Address`) => `Promise`<`Balance`\> |
| `getChannels` | () => `HoprChannels` |
| `getGenesisBlock` | () => `number` |
| `getInfo` | () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprNetworkRegistryAddress`: `string` = hoprNetworkRegistryDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } |
| `getLatestBlockNumber` | () => `Promise`<`number`\> |
| `getNativeBalance` | (`accountAddress`: `Address`) => `Promise`<`Balance`\> |
| `getNetworkRegistry` | () => `HoprNetworkRegistry` |
| `getPrivateKey` | () => `Uint8Array` |
| `getPublicKey` | () => `PublicKey` |
| `getTimestamp` | (`blockNumber`: `number`) => `Promise`<`number`\> |
| `getToken` | () => `HoprToken` |
| `getTransactionsInBlock` | (`blockNumber`: `number`) => `Promise`<`string`[]\> |
| `initiateChannelClosure` | (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `redeemTicket` | (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `sendTransaction` | (`checkDuplicate`: `Boolean`, `essentialTxPayload`: `TransactionPayload`, `handleTxListener`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`SendTransactionReturn`\> |
| `setCommitment` | (`counterparty`: `Address`, `commitment`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `subscribeBlock` | (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` |
| `subscribeError` | (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` |
| `unsubscribe` | () => `void` |
| `updateConfirmedTransaction` | (`hash`: `string`) => `void` |
| `waitUntilReady` | () => `Promise`<`Network`\> |
| `withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:98](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L98)

___

### db

• `Private` **db**: `HoprDB`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:109](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L109)

___

### genesisBlock

• `Private` **genesisBlock**: `number`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:99](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L99)

___

### lastSnapshot

• `Private` **lastSnapshot**: `IndexerSnapshot`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:100](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L100)

___

### latestBlock

• **latestBlock**: `number` = `0`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:92](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L92)

___

### maxConfirmations

• `Private` **maxConfirmations**: `number`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:110](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L110)

___

### startupBlock

• **startupBlock**: `number` = `0`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:93](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L93)

___

### status

• **status**: `IndexerStatus` = `IndexerStatus.STOPPED`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:91](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L91)

___

### unconfirmedEvents

• `Private` **unconfirmedEvents**: `FIFO`<`TypedEvent`<`any`, `any`\>\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:96](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L96)

___

### unsubscribeBlock

• `Private` **unsubscribeBlock**: () => `void`

#### Type declaration

▸ (): `void`

##### Returns

`void`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:105](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L105)

___

### unsubscribeErrors

• `Private` **unsubscribeErrors**: () => `void`

#### Type declaration

▸ (): `void`

##### Returns

`void`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:104](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L104)

## Methods

### addListener

▸ **addListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `IndexerEventNames` |
| `listener` | () => `void` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).addListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:76](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L76)

▸ **addListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"block"`` |
| `listener` | `BlockListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).addListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:77](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L77)

▸ **addListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"block-processed"`` |
| `listener` | `BlockProcessedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).addListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:78](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L78)

▸ **addListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"status"`` |
| `listener` | `StatusListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).addListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:79](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L79)

▸ **addListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"peer"`` |
| `listener` | `PeerListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).addListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:80](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L80)

▸ **addListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `ChannelUpdateEventNames` |
| `listener` | `ChannelUpdateListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).addListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:81](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L81)

▸ **addListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `IndexerEvents` |
| `listener` | `IndexerEventsListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).addListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:82](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L82)

▸ **addListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"network-registry-eligibility-changed"`` |
| `listener` | `NetworkRegistryEligibilityChangedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).addListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:83](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L83)

▸ **addListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"network-registry-status-changed"`` |
| `listener` | `NetworkRegistryStatusChangedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).addListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:87](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L87)

___

### emit

▸ **emit**(`event`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `IndexerEventNames` |

#### Returns

`boolean`

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).emit

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:89](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L89)

▸ **emit**(`event`, `block`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"block"`` |
| `block` | `number` |

#### Returns

`boolean`

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).emit

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:90](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L90)

▸ **emit**(`event`, `block`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"block-processed"`` |
| `block` | `number` |

#### Returns

`boolean`

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).emit

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:91](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L91)

▸ **emit**(`event`, `status`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"status"`` |
| `status` | `IndexerStatus` |

#### Returns

`boolean`

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).emit

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:92](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L92)

▸ **emit**(`event`, `peerData`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"peer"`` |
| `peerData` | `Object` |
| `peerData.id` | `PeerId` |
| `peerData.multiaddrs` | `Multiaddr`[] |

#### Returns

`boolean`

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).emit

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:93](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L93)

▸ **emit**(`event`, `channel`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `ChannelUpdateEventNames` |
| `channel` | [`ChannelEntry`](ChannelEntry.md) |

#### Returns

`boolean`

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).emit

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:94](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L94)

▸ **emit**(`event`, `txHash`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `IndexerEvents` |
| `txHash` | `string` |

#### Returns

`boolean`

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).emit

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:95](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L95)

▸ **emit**(`event`, `account`, `hoprNodes`, `eligibility`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"network-registry-eligibility-changed"`` |
| `account` | `Address` |
| `hoprNodes` | `PublicKey`[] |
| `eligibility` | `boolean` |

#### Returns

`boolean`

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).emit

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:96](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L96)

▸ **emit**(`event`, `isEnabled`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"network-registry-status-changed"`` |
| `isEnabled` | `boolean` |

#### Returns

`boolean`

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).emit

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:102](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L102)

___

### getAccount

▸ **getAccount**(`address`): `Promise`<`AccountEntry`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `Address` |

#### Returns

`Promise`<`AccountEntry`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:996](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L996)

___

### getAddressesAnnouncedOnChain

▸ **getAddressesAnnouncedOnChain**(): `AsyncGenerator`<`Multiaddr`, `void`, `unknown`\>

#### Returns

`AsyncGenerator`<`Multiaddr`, `void`, `unknown`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:1008](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L1008)

___

### getEvents

▸ `Private` **getEvents**(`fromBlock`, `toBlock`, `fetchTokenTransactions?`): `Promise`<{ `events`: `TypedEvent`<`any`, `any`\>[] ; `success`: ``true``  } \| { `success`: ``false``  }\>

Gets all interesting on-chain events, such as Transfer events and payment
channel events

#### Parameters

| Name | Type | Default value | Description |
| :------ | :------ | :------ | :------ |
| `fromBlock` | `number` | `undefined` | block to start from |
| `toBlock` | `number` | `undefined` | last block (inclusive) to consider towards or from the node towards someone else |
| `fetchTokenTransactions` | `boolean` | `false` | - |

#### Returns

`Promise`<{ `events`: `TypedEvent`<`any`, `any`\>[] ; `success`: ``true``  } \| { `success`: ``false``  }\>

all relevant events in the specified block range

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:252](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L252)

___

### getOpenChannelsFrom

▸ **getOpenChannelsFrom**(`source`): `Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

Returns peer's open channels.
NOTE: channels with status 'PENDING_TO_CLOSE' are not included

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `source` | `PublicKey` | peer |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

peer's open channels

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:1054](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L1054)

___

### getPublicKeyOf

▸ **getPublicKeyOf**(`address`): `Promise`<`PublicKey`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `Address` |

#### Returns

`Promise`<`PublicKey`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:1000](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L1000)

___

### getPublicNodes

▸ **getPublicNodes**(): `Promise`<{ `id`: `PeerId` ; `multiaddrs`: `Multiaddr`[]  }[]\>

#### Returns

`Promise`<{ `id`: `PeerId` ; `multiaddrs`: `Multiaddr`[]  }[]\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:1014](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L1014)

___

### getRandomOpenChannel

▸ **getRandomOpenChannel**(): `Promise`<[`ChannelEntry`](ChannelEntry.md)\>

Returns a random open channel.
NOTE: channels with status 'PENDING_TO_CLOSE' are not included

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)\>

an open channel

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:1037](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L1037)

___

### indexEvent

▸ `Private` **indexEvent**(`indexerEvent`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `indexerEvent` | `IndexerEvents` |

#### Returns

`void`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:991](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L991)

___

### listeners

▸ **listeners**(`event`): `Function`[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `IndexerEventNames` |

#### Returns

`Function`[]

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).listeners

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:176](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L176)

___

### off

▸ **off**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `IndexerEventNames` |
| `listener` | () => `void` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).off

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:166](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L166)

▸ **off**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"block"`` |
| `listener` | `BlockListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).off

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:167](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L167)

▸ **off**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"block-processed"`` |
| `listener` | `BlockProcessedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).off

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:168](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L168)

▸ **off**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"status"`` |
| `listener` | `StatusListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).off

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:169](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L169)

▸ **off**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"peer"`` |
| `listener` | `PeerListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).off

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:170](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L170)

▸ **off**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `ChannelUpdateEventNames` |
| `listener` | `ChannelUpdateListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).off

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:171](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L171)

▸ **off**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `IndexerEvents` |
| `listener` | `IndexerEventsListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).off

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:172](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L172)

▸ **off**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"network-registry-eligibility-changed"`` |
| `listener` | `NetworkRegistryEligibilityChangedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).off

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:173](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L173)

▸ **off**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"network-registry-status-changed"`` |
| `listener` | `NetworkRegistryStatusChangedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).off

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:174](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L174)

___

### on

▸ **on**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `IndexerEventNames` |
| `listener` | () => `void` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).on

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:104](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L104)

▸ **on**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"block"`` |
| `listener` | `BlockListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).on

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:105](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L105)

▸ **on**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"block-processed"`` |
| `listener` | `BlockProcessedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).on

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:106](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L106)

▸ **on**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"status"`` |
| `listener` | `StatusListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).on

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:107](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L107)

▸ **on**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"peer"`` |
| `listener` | `PeerListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).on

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:108](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L108)

▸ **on**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `ChannelUpdateEventNames` |
| `listener` | `ChannelUpdateListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).on

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:109](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L109)

▸ **on**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `IndexerEvents` |
| `listener` | `IndexerEventsListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).on

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:110](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L110)

▸ **on**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"network-registry-eligibility-changed"`` |
| `listener` | `NetworkRegistryEligibilityChangedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).on

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:111](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L111)

▸ **on**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"network-registry-status-changed"`` |
| `listener` | `NetworkRegistryStatusChangedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).on

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:112](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L112)

___

### onAnnouncement

▸ `Private` **onAnnouncement**(`event`, `blockNumber`, `lastSnapshot`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `AnnouncementEvent` |
| `blockNumber` | `BN` |
| `lastSnapshot` | `Snapshot` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:809](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L809)

___

### onChannelClosed

▸ `Private` **onChannelClosed**(`channel`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channel` | [`ChannelEntry`](ChannelEntry.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:916](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L916)

___

### onChannelUpdated

▸ `Private` **onChannelUpdated**(`event`, `lastSnapshot`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `ChannelUpdatedEvent` |
| `lastSnapshot` | `Snapshot` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:839](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L839)

___

### onDeregistered

▸ `Private` **onDeregistered**(`event`, `lastSnapshot`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `DeregisteredEvent` \| `DeregisteredByOwnerEvent` |
| `lastSnapshot` | `Snapshot` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:952](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L952)

___

### onEligibilityUpdated

▸ `Private` **onEligibilityUpdated**(`event`, `lastSnapshot`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `EligibilityUpdatedEvent` |
| `lastSnapshot` | `Snapshot` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:921](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L921)

___

### onEnabledNetworkRegistry

▸ `Private` **onEnabledNetworkRegistry**(`event`, `lastSnapshot`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `EnabledNetworkRegistryEvent` |
| `lastSnapshot` | `Snapshot` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:972](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L972)

___

### onNewBlock

▸ `Private` **onNewBlock**(`blockNumber`, `fetchEvents?`, `fetchNativeTxs?`, `blocking?`): `Promise`<`void`\>

Called whenever a new block found.
This will update `this.latestBlock`,
and processes events which are within
confirmed blocks.

#### Parameters

| Name | Type | Default value | Description |
| :------ | :------ | :------ | :------ |
| `blockNumber` | `number` | `undefined` | latest on-chain block number |
| `fetchEvents` | `boolean` | `false` | [optional] if true, query provider for events in block |
| `fetchNativeTxs` | `boolean` | `false` | - |
| `blocking` | `boolean` | `false` | - |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:467](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L467)

___

### onNewEvents

▸ `Private` **onNewEvents**(`events`): `void`

Adds new events to the queue of unprocessed events

**`Dev`**

ignores events that have been processed before.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `events` | `TypedEvent`<`any`, `any`\>[] | new unprocessed events |

#### Returns

`void`

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:630](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L630)

___

### onProviderError

▸ `Private` **onProviderError**(`error`): `Promise`<`void`\>

Called whenever there was a provider error.
Will restart the indexer if needed.

#### Parameters

| Name | Type |
| :------ | :------ |
| `error` | `any` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:415](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L415)

___

### onRegistered

▸ `Private` **onRegistered**(`event`, `lastSnapshot`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `RegisteredEvent` \| `RegisteredByOwnerEvent` |
| `lastSnapshot` | `Snapshot` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:935](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L935)

___

### onTicketRedeemed

▸ `Private` **onTicketRedeemed**(`event`, `lastSnapshot`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `TicketRedeemedEvent` |
| `lastSnapshot` | `Snapshot` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:883](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L883)

___

### onTransfer

▸ `Private` **onTransfer**(`event`, `lastSnapshot`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `TransferEvent` |
| `lastSnapshot` | `Snapshot` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:980](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L980)

___

### once

▸ **once**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `IndexerEventNames` |
| `listener` | () => `void` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).once

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:114](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L114)

▸ **once**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"block"`` |
| `listener` | `BlockListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).once

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:115](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L115)

▸ **once**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"block-processed"`` |
| `listener` | `BlockProcessedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).once

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:116](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L116)

▸ **once**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"status"`` |
| `listener` | `StatusListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).once

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:117](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L117)

▸ **once**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"peer"`` |
| `listener` | `PeerListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).once

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:118](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L118)

▸ **once**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `ChannelUpdateEventNames` |
| `listener` | `ChannelUpdateListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).once

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:119](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L119)

▸ **once**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `IndexerEvents` |
| `listener` | `IndexerEventsListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).once

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:120](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L120)

▸ **once**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"network-registry-eligibility-changed"`` |
| `listener` | `NetworkRegistryEligibilityChangedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).once

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:121](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L121)

▸ **once**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"network-registry-status-changed"`` |
| `listener` | `NetworkRegistryStatusChangedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).once

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:122](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L122)

___

### prependListener

▸ **prependListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `IndexerEventNames` |
| `listener` | () => `void` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:124](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L124)

▸ **prependListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"block"`` |
| `listener` | `BlockListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:125](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L125)

▸ **prependListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"block-processed"`` |
| `listener` | `BlockProcessedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:126](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L126)

▸ **prependListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"status"`` |
| `listener` | `StatusListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:127](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L127)

▸ **prependListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"peer"`` |
| `listener` | `PeerListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:128](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L128)

▸ **prependListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `ChannelUpdateEventNames` |
| `listener` | `ChannelUpdateListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:129](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L129)

▸ **prependListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `IndexerEvents` |
| `listener` | `IndexerEventsListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:130](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L130)

▸ **prependListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"network-registry-eligibility-changed"`` |
| `listener` | `NetworkRegistryEligibilityChangedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:131](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L131)

▸ **prependListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"network-registry-status-changed"`` |
| `listener` | `NetworkRegistryStatusChangedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:135](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L135)

___

### prependOnceListener

▸ **prependOnceListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `IndexerEventNames` |
| `listener` | () => `void` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependOnceListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:137](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L137)

▸ **prependOnceListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"block"`` |
| `listener` | `BlockListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependOnceListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:138](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L138)

▸ **prependOnceListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"block-processed"`` |
| `listener` | `BlockProcessedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependOnceListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:139](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L139)

▸ **prependOnceListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"status"`` |
| `listener` | `StatusListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependOnceListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:140](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L140)

▸ **prependOnceListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"peer"`` |
| `listener` | `PeerListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependOnceListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:141](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L141)

▸ **prependOnceListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `ChannelUpdateEventNames` |
| `listener` | `ChannelUpdateListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependOnceListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:142](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L142)

▸ **prependOnceListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `IndexerEvents` |
| `listener` | `IndexerEventsListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependOnceListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:143](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L143)

▸ **prependOnceListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"network-registry-eligibility-changed"`` |
| `listener` | `NetworkRegistryEligibilityChangedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependOnceListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:144](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L144)

▸ **prependOnceListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"network-registry-status-changed"`` |
| `listener` | `NetworkRegistryStatusChangedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).prependOnceListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:148](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L148)

___

### processPastEvents

▸ `Private` **processPastEvents**(`fromBlock`, `maxToBlock`, `maxBlockRange`): `Promise`<`number`\>

Query past events, this will loop until it gets all blocks from `toBlock` to `fromBlock`.
If we exceed response pull limit, we switch into quering smaller chunks.
TODO: optimize DB and fetch requests

#### Parameters

| Name | Type |
| :------ | :------ |
| `fromBlock` | `number` |
| `maxToBlock` | `number` |
| `maxBlockRange` | `number` |

#### Returns

`Promise`<`number`\>

past events and last queried block

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:367](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L367)

___

### processUnconfirmedEvents

▸ **processUnconfirmedEvents**(`blockNumber`, `lastDatabaseSnapshot`, `blocking`): `Promise`<`void`\>

Process all stored but not yet processed events up to latest
confirmed block (latestBlock - confirmationTime)

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `blockNumber` | `number` | latest on-chain block number |
| `lastDatabaseSnapshot` | `Snapshot` | latest snapshot in database |
| `blocking` | `boolean` | - |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:687](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L687)

___

### removeListener

▸ **removeListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `IndexerEventNames` |
| `listener` | () => `void` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).removeListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:153](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L153)

▸ **removeListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"block"`` |
| `listener` | `BlockListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).removeListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:154](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L154)

▸ **removeListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"block-processed"`` |
| `listener` | `BlockProcessedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).removeListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:155](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L155)

▸ **removeListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"status"`` |
| `listener` | `StatusListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).removeListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:156](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L156)

▸ **removeListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"peer"`` |
| `listener` | `PeerListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).removeListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:157](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L157)

▸ **removeListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `ChannelUpdateEventNames` |
| `listener` | `ChannelUpdateListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).removeListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:158](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L158)

▸ **removeListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `IndexerEvents` |
| `listener` | `IndexerEventsListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).removeListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:159](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L159)

▸ **removeListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"network-registry-eligibility-changed"`` |
| `listener` | `NetworkRegistryEligibilityChangedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).removeListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:160](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L160)

▸ **removeListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | ``"network-registry-status-changed"`` |
| `listener` | `NetworkRegistryStatusChangedListener` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

(EventEmitter as new () =\> IndexerEventEmitter).removeListener

#### Defined in

[packages/core-ethereum/src/indexer/types.ts:164](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/types.ts#L164)

___

### resolvePendingTransaction

▸ **resolvePendingTransaction**(`eventType`, `tx`): `DeferType`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventType` | `IndexerEvents` |
| `tx` | `string` |

#### Returns

`DeferType`<`string`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:1060](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L1060)

___

### restart

▸ `Protected` **restart**(): `Promise`<`void`\>

Restarts the indexer

#### Returns

`Promise`<`void`\>

a promise that resolves once the indexer
has been restarted

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:224](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L224)

___

### start

▸ **start**(`chain`, `genesisBlock`): `Promise`<`void`\>

Starts indexing.

#### Parameters

| Name | Type |
| :------ | :------ |
| `chain` | `Object` |
| `chain.announce` | (`multiaddr`: `Multiaddr`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `chain.finalizeChannelClosure` | (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `chain.fundChannel` | (`partyA`: `Address`, `partyB`: `Address`, `fundsA`: `Balance`, `fundsB`: `Balance`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `chain.getAllQueuingTransactionRequests` | () => `TransactionRequest`[] |
| `chain.getAllUnconfirmedHash` | () => `string`[] |
| `chain.getBalance` | (`accountAddress`: `Address`) => `Promise`<`Balance`\> |
| `chain.getChannels` | () => `HoprChannels` |
| `chain.getGenesisBlock` | () => `number` |
| `chain.getInfo` | () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` = hoprChannelsDeployment.address; `hoprNetworkRegistryAddress`: `string` = hoprNetworkRegistryDeployment.address; `hoprTokenAddress`: `string` = hoprTokenDeployment.address; `network`: `string` = networkInfo.network } |
| `chain.getLatestBlockNumber` | () => `Promise`<`number`\> |
| `chain.getNativeBalance` | (`accountAddress`: `Address`) => `Promise`<`Balance`\> |
| `chain.getNetworkRegistry` | () => `HoprNetworkRegistry` |
| `chain.getPrivateKey` | () => `Uint8Array` |
| `chain.getPublicKey` | () => `PublicKey` |
| `chain.getTimestamp` | (`blockNumber`: `number`) => `Promise`<`number`\> |
| `chain.getToken` | () => `HoprToken` |
| `chain.getTransactionsInBlock` | (`blockNumber`: `number`) => `Promise`<`string`[]\> |
| `chain.initiateChannelClosure` | (`counterparty`: `Address`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `chain.redeemTicket` | (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `chain.sendTransaction` | (`checkDuplicate`: `Boolean`, `essentialTxPayload`: `TransactionPayload`, `handleTxListener`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`SendTransactionReturn`\> |
| `chain.setCommitment` | (`counterparty`: `Address`, `commitment`: `Hash`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `chain.subscribeBlock` | (`cb`: (`blockNumber`: `number`) => `void` \| `Promise`<`void`\>) => () => `void` |
| `chain.subscribeError` | (`cb`: (`err`: `any`) => `void` \| `Promise`<`void`\>) => () => `void` |
| `chain.unsubscribe` | () => `void` |
| `chain.updateConfirmedTransaction` | (`hash`: `string`) => `void` |
| `chain.waitUntilReady` | () => `Promise`<`Network`\> |
| `chain.withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`, `txHandler`: (`tx`: `string`) => `DeferType`<`string`\>) => `Promise`<`string`\> |
| `genesisBlock` | `number` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:121](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L121)

___

### stop

▸ **stop**(): `Promise`<`void`\>

Stops indexing.

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core-ethereum/src/indexer/index.ts:202](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L202)
