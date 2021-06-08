[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / Indexer

# Class: Indexer

Indexes HoprChannels smart contract and stores to the DB,
all channels in the network.
Also keeps track of the latest block number.

## Hierarchy

- `EventEmitter`

  ↳ **Indexer**

## Table of contents

### Constructors

- [constructor](indexer.md#constructor)

### Properties

- [address](indexer.md#address)
- [latestBlock](indexer.md#latestblock)
- [pendingCommitments](indexer.md#pendingcommitments)
- [status](indexer.md#status)
- [unconfirmedEvents](indexer.md#unconfirmedevents)
- [captureRejectionSymbol](indexer.md#capturerejectionsymbol)
- [captureRejections](indexer.md#capturerejections)
- [defaultMaxListeners](indexer.md#defaultmaxlisteners)
- [errorMonitor](indexer.md#errormonitor)

### Methods

- [addListener](indexer.md#addlistener)
- [emit](indexer.md#emit)
- [eventNames](indexer.md#eventnames)
- [getAccount](indexer.md#getaccount)
- [getAnnouncedAddresses](indexer.md#getannouncedaddresses)
- [getChannel](indexer.md#getchannel)
- [getChannels](indexer.md#getchannels)
- [getChannelsOf](indexer.md#getchannelsof)
- [getMaxListeners](indexer.md#getmaxlisteners)
- [getOpenRoutingChannelsFromPeer](indexer.md#getopenroutingchannelsfrompeer)
- [getPublicKeyOf](indexer.md#getpublickeyof)
- [getPublicNodes](indexer.md#getpublicnodes)
- [getRandomChannel](indexer.md#getrandomchannel)
- [listenerCount](indexer.md#listenercount)
- [listeners](indexer.md#listeners)
- [off](indexer.md#off)
- [on](indexer.md#on)
- [onAnnouncement](indexer.md#onannouncement)
- [onChannelUpdated](indexer.md#onchannelupdated)
- [onNewBlock](indexer.md#onnewblock)
- [onNewEvents](indexer.md#onnewevents)
- [onOwnUnsetCommitment](indexer.md#onownunsetcommitment)
- [once](indexer.md#once)
- [prependListener](indexer.md#prependlistener)
- [prependOnceListener](indexer.md#prependoncelistener)
- [processPastEvents](indexer.md#processpastevents)
- [rawListeners](indexer.md#rawlisteners)
- [removeAllListeners](indexer.md#removealllisteners)
- [removeListener](indexer.md#removelistener)
- [resolveCommitmentPromise](indexer.md#resolvecommitmentpromise)
- [restart](indexer.md#restart)
- [setMaxListeners](indexer.md#setmaxlisteners)
- [start](indexer.md#start)
- [stop](indexer.md#stop)
- [toIndexerChannel](indexer.md#toindexerchannel)
- [waitForCommitment](indexer.md#waitforcommitment)
- [listenerCount](indexer.md#listenercount)
- [on](indexer.md#on)
- [once](indexer.md#once)

## Constructors

### constructor

• **new Indexer**(`genesisBlock`, `db`, `chain`, `maxConfirmations`, `blockRange`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `genesisBlock` | `number` |
| `db` | `HoprDB` |
| `chain` | `Object` |
| `chain.announce` | (`multiaddr`: `Multiaddr`) => `Promise`<string\> |
| `chain.finalizeChannelClosure` | (`counterparty`: `Address`) => `Promise`<string\> |
| `chain.fundChannel` | (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`) => `Promise`<string\> |
| `chain.getBalance` | (`address`: `Address`) => `Promise`<Balance\> |
| `chain.getChannels` | () => `HoprChannels` |
| `chain.getGenesisBlock` | () => `number` |
| `chain.getInfo` | () => `string` |
| `chain.getLatestBlockNumber` | () => `Promise`<number\> |
| `chain.getNativeBalance` | (`address`: `Address`) => `Promise`<NativeBalance\> |
| `chain.getPrivateKey` | () => `Uint8Array` |
| `chain.getPublicKey` | () => `PublicKey` |
| `chain.getWallet` | () => `Wallet` |
| `chain.initiateChannelClosure` | (`counterparty`: `Address`) => `Promise`<string\> |
| `chain.openChannel` | (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`) => `Promise`<string\> |
| `chain.redeemTicket` | (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`) => `Promise`<string\> |
| `chain.setCommitment` | (`counterparty`: `Address`, `comm`: `Hash`) => `Promise`<string\> |
| `chain.subscribeBlock` | (`cb`: `any`) => `JsonRpcProvider` \| `WebSocketProvider` |
| `chain.subscribeChannelEvents` | (`cb`: `any`) => `HoprChannels` |
| `chain.subscribeError` | (`cb`: `any`) => `void` |
| `chain.unsubscribe` | () => `void` |
| `chain.waitUntilReady` | () => `Promise`<Network\> |
| `chain.withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`) => `Promise`<string\> |
| `maxConfirmations` | `number` |
| `blockRange` | `number` |

#### Overrides

EventEmitter.constructor

#### Defined in

[core-ethereum/src/indexer/index.ts:31](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L31)

## Properties

### address

• `Private` **address**: `Address`

#### Defined in

[core-ethereum/src/indexer/index.ts:30](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L30)

___

### latestBlock

• **latestBlock**: `number` = 0

#### Defined in

[core-ethereum/src/indexer/index.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L28)

___

### pendingCommitments

• `Private` **pendingCommitments**: `Map`<string, DeferredPromise<void\>\>

#### Defined in

[core-ethereum/src/indexer/index.ts:31](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L31)

___

### status

• **status**: ``"started"`` \| ``"restarting"`` \| ``"stopped"`` = 'stopped'

#### Defined in

[core-ethereum/src/indexer/index.ts:27](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L27)

___

### unconfirmedEvents

• `Private` **unconfirmedEvents**: `Heap`<Event<any\>\>

#### Defined in

[core-ethereum/src/indexer/index.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L29)

___

### captureRejectionSymbol

▪ `Static` `Readonly` **captureRejectionSymbol**: typeof [captureRejectionSymbol](indexer.md#capturerejectionsymbol)

#### Inherited from

EventEmitter.captureRejectionSymbol

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:43

___

### captureRejections

▪ `Static` **captureRejections**: `boolean`

Sets or gets the default captureRejection value for all emitters.

#### Inherited from

EventEmitter.captureRejections

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:49

___

### defaultMaxListeners

▪ `Static` **defaultMaxListeners**: `number`

#### Inherited from

EventEmitter.defaultMaxListeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:50

___

### errorMonitor

▪ `Static` `Readonly` **errorMonitor**: typeof [errorMonitor](indexer.md#errormonitor)

This symbol shall be used to install a listener for only monitoring `'error'`
events. Listeners installed using this symbol are called before the regular
`'error'` listeners are called.

Installing a listener using this symbol does not change the behavior once an
`'error'` event is emitted, therefore the process will still crash if no
regular `'error'` listener is installed.

#### Inherited from

EventEmitter.errorMonitor

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:42

## Methods

### addListener

▸ **addListener**(`event`, `listener`): [Indexer](indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[Indexer](indexer.md)

#### Inherited from

EventEmitter.addListener

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:62

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

core-ethereum/node_modules/@types/node/events.d.ts:72

___

### eventNames

▸ **eventNames**(): (`string` \| `symbol`)[]

#### Returns

(`string` \| `symbol`)[]

#### Inherited from

EventEmitter.eventNames

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:77

___

### getAccount

▸ **getAccount**(`address`): `Promise`<AccountEntry\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `Address` |

#### Returns

`Promise`<AccountEntry\>

#### Defined in

[core-ethereum/src/indexer/index.ts:334](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L334)

___

### getAnnouncedAddresses

▸ **getAnnouncedAddresses**(): `Promise`<Multiaddr[]\>

#### Returns

`Promise`<Multiaddr[]\>

#### Defined in

[core-ethereum/src/indexer/index.ts:377](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L377)

___

### getChannel

▸ **getChannel**(`channelId`): `Promise`<ChannelEntry\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | `Hash` |

#### Returns

`Promise`<ChannelEntry\>

#### Defined in

[core-ethereum/src/indexer/index.ts:338](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L338)

___

### getChannels

▸ **getChannels**(`filter?`): `Promise`<ChannelEntry[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`channel`: `ChannelEntry`) => `boolean` |

#### Returns

`Promise`<ChannelEntry[]\>

#### Defined in

[core-ethereum/src/indexer/index.ts:342](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L342)

___

### getChannelsOf

▸ **getChannelsOf**(`address`): `Promise`<ChannelEntry[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `Address` |

#### Returns

`Promise`<ChannelEntry[]\>

#### Defined in

[core-ethereum/src/indexer/index.ts:346](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L346)

___

### getMaxListeners

▸ **getMaxListeners**(): `number`

#### Returns

`number`

#### Inherited from

EventEmitter.getMaxListeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:69

___

### getOpenRoutingChannelsFromPeer

▸ **getOpenRoutingChannelsFromPeer**(`source`): `Promise`<[RoutingChannel](../modules.md#routingchannel)[]\>

Returns peer's open channels.
NOTE: channels with status 'PENDING_TO_CLOSE' are not included

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `source` | `PeerId` | peer |

#### Returns

`Promise`<[RoutingChannel](../modules.md#routingchannel)[]\>

peer's open channels

#### Defined in

[core-ethereum/src/indexer/index.ts:407](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L407)

___

### getPublicKeyOf

▸ **getPublicKeyOf**(`address`): `Promise`<PublicKey\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `Address` |

#### Returns

`Promise`<PublicKey\>

#### Defined in

[core-ethereum/src/indexer/index.ts:353](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L353)

___

### getPublicNodes

▸ **getPublicNodes**(): `Promise`<Multiaddr[]\>

#### Returns

`Promise`<Multiaddr[]\>

#### Defined in

[core-ethereum/src/indexer/index.ts:381](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L381)

___

### getRandomChannel

▸ **getRandomChannel**(): `Promise`<[RoutingChannel](../modules.md#routingchannel)\>

#### Returns

`Promise`<[RoutingChannel](../modules.md#routingchannel)\>

#### Defined in

[core-ethereum/src/indexer/index.ts:387](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L387)

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

core-ethereum/node_modules/@types/node/events.d.ts:73

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

core-ethereum/node_modules/@types/node/events.d.ts:70

___

### off

▸ **off**(`event`, `listener`): [Indexer](indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[Indexer](indexer.md)

#### Inherited from

EventEmitter.off

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:66

___

### on

▸ **on**(`event`, `listener`): [Indexer](indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[Indexer](indexer.md)

#### Inherited from

EventEmitter.on

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:63

___

### onAnnouncement

▸ `Private` **onAnnouncement**(`event`, `blockNumber`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `Event`<``"Announcement"``\> |
| `blockNumber` | `BN` |

#### Returns

`Promise`<void\>

#### Defined in

[core-ethereum/src/indexer/index.ts:253](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L253)

___

### onChannelUpdated

▸ `Private` **onChannelUpdated**(`event`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `Event`<``"ChannelUpdate"``\> |

#### Returns

`Promise`<void\>

#### Defined in

[core-ethereum/src/indexer/index.ts:277](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L277)

___

### onNewBlock

▸ `Private` **onNewBlock**(`blockNumber`): `Promise`<void\>

Called whenever a new block found.
This will update {this.latestBlock},
and processes events which are within
confirmed blocks.

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockNumber` | `number` |

#### Returns

`Promise`<void\>

#### Defined in

[core-ethereum/src/indexer/index.ts:191](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L191)

___

### onNewEvents

▸ `Private` **onNewEvents**(`events`): `void`

Called whenever we receive new events.

#### Parameters

| Name | Type |
| :------ | :------ |
| `events` | `Event`<any\>[] |

#### Returns

`void`

#### Defined in

[core-ethereum/src/indexer/index.ts:249](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L249)

___

### onOwnUnsetCommitment

▸ `Private` **onOwnUnsetCommitment**(`channel`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channel` | `ChannelEntry` |

#### Returns

`Promise`<void\>

#### Defined in

[core-ethereum/src/indexer/index.ts:302](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L302)

___

### once

▸ **once**(`event`, `listener`): [Indexer](indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[Indexer](indexer.md)

#### Inherited from

EventEmitter.once

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:64

___

### prependListener

▸ **prependListener**(`event`, `listener`): [Indexer](indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[Indexer](indexer.md)

#### Inherited from

EventEmitter.prependListener

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:75

___

### prependOnceListener

▸ **prependOnceListener**(`event`, `listener`): [Indexer](indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[Indexer](indexer.md)

#### Inherited from

EventEmitter.prependOnceListener

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:76

___

### processPastEvents

▸ `Private` **processPastEvents**(`fromBlock`, `maxToBlock`, `maxBlockRange`): `Promise`<number\>

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

`Promise`<number\>

past events and last queried block

#### Defined in

[core-ethereum/src/indexer/index.ts:137](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L137)

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

core-ethereum/node_modules/@types/node/events.d.ts:71

___

### removeAllListeners

▸ **removeAllListeners**(`event?`): [Indexer](indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event?` | `string` \| `symbol` |

#### Returns

[Indexer](indexer.md)

#### Inherited from

EventEmitter.removeAllListeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:67

___

### removeListener

▸ **removeListener**(`event`, `listener`): [Indexer](indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[Indexer](indexer.md)

#### Inherited from

EventEmitter.removeListener

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:65

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

[core-ethereum/src/indexer/index.ts:330](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L330)

___

### restart

▸ `Private` **restart**(): `Promise`<void\>

#### Returns

`Promise`<void\>

#### Defined in

[core-ethereum/src/indexer/index.ts:112](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L112)

___

### setMaxListeners

▸ **setMaxListeners**(`n`): [Indexer](indexer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `n` | `number` |

#### Returns

[Indexer](indexer.md)

#### Inherited from

EventEmitter.setMaxListeners

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:68

___

### start

▸ **start**(): `Promise`<void\>

Starts indexing.

#### Returns

`Promise`<void\>

#### Defined in

[core-ethereum/src/indexer/index.ts:49](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L49)

___

### stop

▸ **stop**(): `Promise`<void\>

Stops indexing.

#### Returns

`Promise`<void\>

#### Defined in

[core-ethereum/src/indexer/index.ts:102](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L102)

___

### toIndexerChannel

▸ `Private` **toIndexerChannel**(`source`, `channel`): `Promise`<[RoutingChannel](../modules.md#routingchannel)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `source` | `PeerId` |
| `channel` | `ChannelEntry` |

#### Returns

`Promise`<[RoutingChannel](../modules.md#routingchannel)\>

#### Defined in

[core-ethereum/src/indexer/index.ts:362](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L362)

___

### waitForCommitment

▸ **waitForCommitment**(`channelId`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | `Hash` |

#### Returns

`Promise`<void\>

#### Defined in

[core-ethereum/src/indexer/index.ts:318](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L318)

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

core-ethereum/node_modules/@types/node/events.d.ts:31

___

### on

▸ `Static` **on**(`emitter`, `event`): `AsyncIterableIterator`<any\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `EventEmitter` |
| `event` | `string` |

#### Returns

`AsyncIterableIterator`<any\>

#### Inherited from

EventEmitter.on

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:28

___

### once

▸ `Static` **once**(`emitter`, `event`): `Promise`<any[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `NodeEventTarget` |
| `event` | `string` \| `symbol` |

#### Returns

`Promise`<any[]\>

#### Inherited from

EventEmitter.once

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:26

▸ `Static` **once**(`emitter`, `event`): `Promise`<any[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `DOMEventTarget` |
| `event` | `string` |

#### Returns

`Promise`<any[]\>

#### Inherited from

EventEmitter.once

#### Defined in

core-ethereum/node_modules/@types/node/events.d.ts:27
