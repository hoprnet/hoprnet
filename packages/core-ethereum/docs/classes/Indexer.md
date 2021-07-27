[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / Indexer

# Class: Indexer

Indexes HoprChannels smart contract and stores to the DB,
all channels in the network.
Also keeps track of the latest block number.

## Hierarchy

- `EventEmitter`

  ↳ **`Indexer`**

## Table of contents

### Constructors

- [constructor](Indexer.md#constructor)

### Properties

- [chain](Indexer.md#chain)
- [genesisBlock](Indexer.md#genesisblock)
- [latestBlock](Indexer.md#latestblock)
- [pendingCommitments](Indexer.md#pendingcommitments)
- [status](Indexer.md#status)
- [unconfirmedEvents](Indexer.md#unconfirmedevents)
- [captureRejectionSymbol](Indexer.md#capturerejectionsymbol)
- [captureRejections](Indexer.md#capturerejections)
- [defaultMaxListeners](Indexer.md#defaultmaxlisteners)
- [errorMonitor](Indexer.md#errormonitor)

### Methods

- [addListener](Indexer.md#addlistener)
- [emit](Indexer.md#emit)
- [eventNames](Indexer.md#eventnames)
- [getAccount](Indexer.md#getaccount)
- [getAnnouncedAddresses](Indexer.md#getannouncedaddresses)
- [getChannel](Indexer.md#getchannel)
- [getChannels](Indexer.md#getchannels)
- [getChannelsFrom](Indexer.md#getchannelsfrom)
- [getChannelsTo](Indexer.md#getchannelsto)
- [getMaxListeners](Indexer.md#getmaxlisteners)
- [getOpenChannelsFrom](Indexer.md#getopenchannelsfrom)
- [getPublicKeyOf](Indexer.md#getpublickeyof)
- [getPublicNodes](Indexer.md#getpublicnodes)
- [getRandomOpenChannel](Indexer.md#getrandomopenchannel)
- [listenerCount](Indexer.md#listenercount)
- [listeners](Indexer.md#listeners)
- [off](Indexer.md#off)
- [on](Indexer.md#on)
- [onAnnouncement](Indexer.md#onannouncement)
- [onChannelUpdated](Indexer.md#onchannelupdated)
- [onNewBlock](Indexer.md#onnewblock)
- [onNewEvents](Indexer.md#onnewevents)
- [onOwnUnsetCommitment](Indexer.md#onownunsetcommitment)
- [once](Indexer.md#once)
- [prependListener](Indexer.md#prependlistener)
- [prependOnceListener](Indexer.md#prependoncelistener)
- [processPastEvents](Indexer.md#processpastevents)
- [rawListeners](Indexer.md#rawlisteners)
- [removeAllListeners](Indexer.md#removealllisteners)
- [removeListener](Indexer.md#removelistener)
- [resolveCommitmentPromise](Indexer.md#resolvecommitmentpromise)
- [restart](Indexer.md#restart)
- [setMaxListeners](Indexer.md#setmaxlisteners)
- [start](Indexer.md#start)
- [stop](Indexer.md#stop)
- [waitForCommitment](Indexer.md#waitforcommitment)
- [getEventListener](Indexer.md#geteventlistener)
- [listenerCount](Indexer.md#listenercount)
- [on](Indexer.md#on)
- [once](Indexer.md#once)

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

EventEmitter.constructor

#### Defined in

[core-ethereum/src/indexer/index.ts:33](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L33)

## Properties

### chain

• `Private` **chain**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `announce` | (`multiaddr`: `Multiaddr`) => `Promise`<`string`\> |
| `finalizeChannelClosure` | (`counterparty`: `Address`) => `Promise`<`string`\> |
| `fundChannel` | (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`) => `Promise`<`string`\> |
| `getBalance` | (`address`: `Address`) => `Promise`<`Balance`\> |
| `getChannels` | () => `HoprChannels` |
| `getGenesisBlock` | () => `number` |
| `getInfo` | () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` ; `hoprTokenAddress`: `string` ; `network`: `Networks`  } |
| `getLatestBlockNumber` | () => `Promise`<`number`\> |
| `getNativeBalance` | (`address`: `Address`) => `Promise`<`NativeBalance`\> |
| `getPrivateKey` | () => `Uint8Array` |
| `getPublicKey` | () => `PublicKey` |
| `getWallet` | () => `Wallet` |
| `initiateChannelClosure` | (`counterparty`: `Address`) => `Promise`<`string`\> |
| `openChannel` | (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`) => `Promise`<`string`\> |
| `redeemTicket` | (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`) => `Promise`<`string`\> |
| `setCommitment` | (`counterparty`: `Address`, `comm`: `Hash`) => `Promise`<`string`\> |
| `subscribeBlock` | (`cb`: `any`) => `JsonRpcProvider` \| `WebSocketProvider` |
| `subscribeChannelEvents` | (`cb`: `any`) => `HoprChannels` |
| `subscribeError` | (`cb`: `any`) => `void` |
| `unsubscribe` | () => `void` |
| `waitUntilReady` | () => `Promise`<`Network`\> |
| `withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`) => `Promise`<`string`\> |

#### Defined in

[core-ethereum/src/indexer/index.ts:30](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L30)

___

### genesisBlock

• `Private` **genesisBlock**: `number`

#### Defined in

[core-ethereum/src/indexer/index.ts:31](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L31)

___

### latestBlock

• **latestBlock**: `number` = `0`

#### Defined in

[core-ethereum/src/indexer/index.ts:27](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L27)

___

### pendingCommitments

• `Private` **pendingCommitments**: `Map`<`string`, `DeferredPromise`<`void`\>\>

#### Defined in

[core-ethereum/src/indexer/index.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L29)

___

### status

• **status**: ``"started"`` \| ``"restarting"`` \| ``"stopped"`` = `'stopped'`

#### Defined in

[core-ethereum/src/indexer/index.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L26)

___

### unconfirmedEvents

• `Private` **unconfirmedEvents**: `Heap`<`Event`<`any`\>\>

#### Defined in

[core-ethereum/src/indexer/index.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L28)

___

### captureRejectionSymbol

▪ `Static` `Readonly` **captureRejectionSymbol**: typeof [`captureRejectionSymbol`](default.md#capturerejectionsymbol)

#### Inherited from

EventEmitter.captureRejectionSymbol

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:46

___

### captureRejections

▪ `Static` **captureRejections**: `boolean`

Sets or gets the default captureRejection value for all emitters.

#### Inherited from

EventEmitter.captureRejections

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:52

___

### defaultMaxListeners

▪ `Static` **defaultMaxListeners**: `number`

#### Inherited from

EventEmitter.defaultMaxListeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:53

___

### errorMonitor

▪ `Static` `Readonly` **errorMonitor**: typeof [`errorMonitor`](default.md#errormonitor)

This symbol shall be used to install a listener for only monitoring `'error'`
events. Listeners installed using this symbol are called before the regular
`'error'` listeners are called.

Installing a listener using this symbol does not change the behavior once an
`'error'` event is emitted, therefore the process will still crash if no
regular `'error'` listener is installed.

#### Inherited from

EventEmitter.errorMonitor

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:45

## Methods

### addListener

▸ **addListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.addListener

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:72

___

### emit

▸ **emit**(`event`, ...`args`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `...args` | `any`[] |

#### Returns

`boolean`

#### Inherited from

EventEmitter.emit

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:82

___

### eventNames

▸ **eventNames**(): (`string` \| `symbol`)[]

#### Returns

(`string` \| `symbol`)[]

#### Inherited from

EventEmitter.eventNames

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:87

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

[core-ethereum/src/indexer/index.ts:351](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L351)

___

### getAnnouncedAddresses

▸ **getAnnouncedAddresses**(): `Promise`<`Multiaddr`[]\>

#### Returns

`Promise`<`Multiaddr`[]\>

#### Defined in

[core-ethereum/src/indexer/index.ts:383](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L383)

___

### getChannel

▸ **getChannel**(`channelId`): `Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | `Hash` |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Defined in

[core-ethereum/src/indexer/index.ts:355](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L355)

___

### getChannels

▸ **getChannels**(`filter?`): `Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`channel`: [`ChannelEntry`](ChannelEntry.md)) => `boolean` |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Defined in

[core-ethereum/src/indexer/index.ts:359](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L359)

___

### getChannelsFrom

▸ **getChannelsFrom**(`address`): `Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `Address` |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Defined in

[core-ethereum/src/indexer/index.ts:363](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L363)

___

### getChannelsTo

▸ **getChannelsTo**(`address`): `Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `Address` |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Defined in

[core-ethereum/src/indexer/index.ts:369](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L369)

___

### getMaxListeners

▸ **getMaxListeners**(): `number`

#### Returns

`number`

#### Inherited from

EventEmitter.getMaxListeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:79

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

[core-ethereum/src/indexer/index.ts:415](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L415)

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

[core-ethereum/src/indexer/index.ts:375](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L375)

___

### getPublicNodes

▸ **getPublicNodes**(): `Promise`<`Multiaddr`[]\>

#### Returns

`Promise`<`Multiaddr`[]\>

#### Defined in

[core-ethereum/src/indexer/index.ts:387](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L387)

___

### getRandomOpenChannel

▸ **getRandomOpenChannel**(): `Promise`<[`ChannelEntry`](ChannelEntry.md)\>

Returns a random open channel.
NOTE: channels with status 'PENDING_TO_CLOSE' are not included

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)\>

an open channel

#### Defined in

[core-ethereum/src/indexer/index.ts:398](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L398)

___

### listenerCount

▸ **listenerCount**(`event`): `number`

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |

#### Returns

`number`

#### Inherited from

EventEmitter.listenerCount

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:83

___

### listeners

▸ **listeners**(`event`): `Function`[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |

#### Returns

`Function`[]

#### Inherited from

EventEmitter.listeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:80

___

### off

▸ **off**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.off

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:76

___

### on

▸ **on**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.on

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:73

___

### onAnnouncement

▸ `Private` **onAnnouncement**(`event`, `blockNumber`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `Event`<``"Announcement"``\> |
| `blockNumber` | `BN` |

#### Returns

`Promise`<`void`\>

#### Defined in

[core-ethereum/src/indexer/index.ts:269](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L269)

___

### onChannelUpdated

▸ `Private` **onChannelUpdated**(`event`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `Event`<``"ChannelUpdate"``\> |

#### Returns

`Promise`<`void`\>

#### Defined in

[core-ethereum/src/indexer/index.ts:293](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L293)

___

### onNewBlock

▸ `Private` **onNewBlock**(`blockNumber`): `Promise`<`void`\>

Called whenever a new block found.
This will update {this.latestBlock},
and processes events which are within
confirmed blocks.

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockNumber` | `number` |

#### Returns

`Promise`<`void`\>

#### Defined in

[core-ethereum/src/indexer/index.ts:192](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L192)

___

### onNewEvents

▸ `Private` **onNewEvents**(`events`): `void`

Called whenever we receive new events.

#### Parameters

| Name | Type |
| :------ | :------ |
| `events` | `Event`<`any`\>[] |

#### Returns

`void`

#### Defined in

[core-ethereum/src/indexer/index.ts:265](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L265)

___

### onOwnUnsetCommitment

▸ `Private` **onOwnUnsetCommitment**(`channel`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channel` | [`ChannelEntry`](ChannelEntry.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[core-ethereum/src/indexer/index.ts:320](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L320)

___

### once

▸ **once**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.once

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:74

___

### prependListener

▸ **prependListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.prependListener

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:85

___

### prependOnceListener

▸ **prependOnceListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.prependOnceListener

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:86

___

### processPastEvents

▸ `Private` **processPastEvents**(`fromBlock`, `maxToBlock`, `maxBlockRange`): `Promise`<`number`\>

Query past events, this will loop until it gets all blocks from {toBlock} to {fromBlock}.
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

[core-ethereum/src/indexer/index.ts:139](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L139)

___

### rawListeners

▸ **rawListeners**(`event`): `Function`[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |

#### Returns

`Function`[]

#### Inherited from

EventEmitter.rawListeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:81

___

### removeAllListeners

▸ **removeAllListeners**(`event?`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event?` | `string` \| `symbol` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.removeAllListeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:77

___

### removeListener

▸ **removeListener**(`event`, `listener`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.removeListener

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:75

___

### resolveCommitmentPromise

▸ `Private` **resolveCommitmentPromise**(`channelId`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | `Hash` |

#### Returns

`void`

#### Defined in

[core-ethereum/src/indexer/index.ts:347](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L347)

___

### restart

▸ `Private` **restart**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

[core-ethereum/src/indexer/index.ts:113](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L113)

___

### setMaxListeners

▸ **setMaxListeners**(`n`): [`Indexer`](Indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `n` | `number` |

#### Returns

[`Indexer`](Indexer.md)

#### Inherited from

EventEmitter.setMaxListeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:78

___

### start

▸ **start**(`chain`, `genesisBlock`): `Promise`<`void`\>

Starts indexing.

#### Parameters

| Name | Type |
| :------ | :------ |
| `chain` | `Object` |
| `chain.announce` | (`multiaddr`: `Multiaddr`) => `Promise`<`string`\> |
| `chain.finalizeChannelClosure` | (`counterparty`: `Address`) => `Promise`<`string`\> |
| `chain.fundChannel` | (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`) => `Promise`<`string`\> |
| `chain.getBalance` | (`address`: `Address`) => `Promise`<`Balance`\> |
| `chain.getChannels` | () => `HoprChannels` |
| `chain.getGenesisBlock` | () => `number` |
| `chain.getInfo` | () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` ; `hoprTokenAddress`: `string` ; `network`: `Networks`  } |
| `chain.getLatestBlockNumber` | () => `Promise`<`number`\> |
| `chain.getNativeBalance` | (`address`: `Address`) => `Promise`<`NativeBalance`\> |
| `chain.getPrivateKey` | () => `Uint8Array` |
| `chain.getPublicKey` | () => `PublicKey` |
| `chain.getWallet` | () => `Wallet` |
| `chain.initiateChannelClosure` | (`counterparty`: `Address`) => `Promise`<`string`\> |
| `chain.openChannel` | (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`) => `Promise`<`string`\> |
| `chain.redeemTicket` | (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`) => `Promise`<`string`\> |
| `chain.setCommitment` | (`counterparty`: `Address`, `comm`: `Hash`) => `Promise`<`string`\> |
| `chain.subscribeBlock` | (`cb`: `any`) => `JsonRpcProvider` \| `WebSocketProvider` |
| `chain.subscribeChannelEvents` | (`cb`: `any`) => `HoprChannels` |
| `chain.subscribeError` | (`cb`: `any`) => `void` |
| `chain.unsubscribe` | () => `void` |
| `chain.waitUntilReady` | () => `Promise`<`Network`\> |
| `chain.withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`) => `Promise`<`string`\> |
| `genesisBlock` | `number` |

#### Returns

`Promise`<`void`\>

#### Defined in

[core-ethereum/src/indexer/index.ts:46](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L46)

___

### stop

▸ **stop**(): `Promise`<`void`\>

Stops indexing.

#### Returns

`Promise`<`void`\>

#### Defined in

[core-ethereum/src/indexer/index.ts:102](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L102)

___

### waitForCommitment

▸ **waitForCommitment**(`channelId`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | `Hash` |

#### Returns

`Promise`<`void`\>

#### Defined in

[core-ethereum/src/indexer/index.ts:335](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L335)

___

### getEventListener

▸ `Static` **getEventListener**(`emitter`, `name`): `Function`[]

Returns a list listener for a specific emitter event name.

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `DOMEventTarget` \| `EventEmitter` |
| `name` | `string` \| `symbol` |

#### Returns

`Function`[]

#### Inherited from

EventEmitter.getEventListener

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:34

___

### listenerCount

▸ `Static` **listenerCount**(`emitter`, `event`): `number`

**`deprecated`** since v4.0.0

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `EventEmitter` |
| `event` | `string` \| `symbol` |

#### Returns

`number`

#### Inherited from

EventEmitter.listenerCount

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:30

___

### on

▸ `Static` **on**(`emitter`, `event`, `options?`): `AsyncIterableIterator`<`any`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `EventEmitter` |
| `event` | `string` |
| `options?` | `StaticEventEmitterOptions` |

#### Returns

`AsyncIterableIterator`<`any`\>

#### Inherited from

EventEmitter.on

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:27

___

### once

▸ `Static` **once**(`emitter`, `event`, `options?`): `Promise`<`any`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `NodeEventTarget` |
| `event` | `string` \| `symbol` |
| `options?` | `StaticEventEmitterOptions` |

#### Returns

`Promise`<`any`[]\>

#### Inherited from

EventEmitter.once

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:25

▸ `Static` **once**(`emitter`, `event`, `options?`): `Promise`<`any`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `DOMEventTarget` |
| `event` | `string` |
| `options?` | `StaticEventEmitterOptions` |

#### Returns

`Promise`<`any`[]\>

#### Inherited from

EventEmitter.once

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:26
