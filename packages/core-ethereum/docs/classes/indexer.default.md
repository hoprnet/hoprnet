[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [indexer](../modules/indexer.md) / default

# Class: default

[indexer](../modules/indexer.md).default

Indexes HoprChannels smart contract and stores to the DB,
all channels in the network.
Also keeps track of the latest block number.

## Hierarchy

- *EventEmitter*

  ↳ **default**

## Table of contents

### Constructors

- [constructor](indexer.default.md#constructor)

### Properties

- [latestBlock](indexer.default.md#latestblock)
- [status](indexer.default.md#status)
- [unconfirmedEvents](indexer.default.md#unconfirmedevents)
- [captureRejectionSymbol](indexer.default.md#capturerejectionsymbol)
- [captureRejections](indexer.default.md#capturerejections)
- [defaultMaxListeners](indexer.default.md#defaultmaxlisteners)
- [errorMonitor](indexer.default.md#errormonitor)

### Methods

- [addListener](indexer.default.md#addlistener)
- [emit](indexer.default.md#emit)
- [eventNames](indexer.default.md#eventnames)
- [getAccount](indexer.default.md#getaccount)
- [getAnnouncedAddresses](indexer.default.md#getannouncedaddresses)
- [getChannel](indexer.default.md#getchannel)
- [getChannels](indexer.default.md#getchannels)
- [getChannelsFromPeer](indexer.default.md#getchannelsfrompeer)
- [getChannelsOf](indexer.default.md#getchannelsof)
- [getMaxListeners](indexer.default.md#getmaxlisteners)
- [getPublicKeyOf](indexer.default.md#getpublickeyof)
- [getPublicNodes](indexer.default.md#getpublicnodes)
- [getRandomChannel](indexer.default.md#getrandomchannel)
- [listenerCount](indexer.default.md#listenercount)
- [listeners](indexer.default.md#listeners)
- [off](indexer.default.md#off)
- [on](indexer.default.md#on)
- [onAnnouncement](indexer.default.md#onannouncement)
- [onChannelUpdated](indexer.default.md#onchannelupdated)
- [onNewBlock](indexer.default.md#onnewblock)
- [onNewEvents](indexer.default.md#onnewevents)
- [once](indexer.default.md#once)
- [prependListener](indexer.default.md#prependlistener)
- [prependOnceListener](indexer.default.md#prependoncelistener)
- [processPastEvents](indexer.default.md#processpastevents)
- [rawListeners](indexer.default.md#rawlisteners)
- [removeAllListeners](indexer.default.md#removealllisteners)
- [removeListener](indexer.default.md#removelistener)
- [restart](indexer.default.md#restart)
- [setMaxListeners](indexer.default.md#setmaxlisteners)
- [start](indexer.default.md#start)
- [stop](indexer.default.md#stop)
- [toIndexerChannel](indexer.default.md#toindexerchannel)
- [listenerCount](indexer.default.md#listenercount)
- [on](indexer.default.md#on)
- [once](indexer.default.md#once)

## Constructors

### constructor

\+ **new default**(`genesisBlock`: *number*, `db`: *HoprDB*, `chain`: { `announce`: (`multiaddr`: Multiaddr) => *Promise*<string\> ; `finalizeChannelClosure`: (`counterparty`: *any*) => *Promise*<string\> ; `fundChannel`: (`me`: *Address*, `counterparty`: *Address*, `myTotal`: *Balance*, `theirTotal`: *Balance*) => *Promise*<string\> ; `getBalance`: (`address`: *Address*) => *Promise*<Balance\> ; `getChannels`: () => [*HoprChannels*](contracts_hoprchannels.hoprchannels.md) ; `getGenesisBlock`: () => *number* ; `getInfo`: () => *string* ; `getLatestBlockNumber`: () => *Promise*<number\> ; `getNativeBalance`: (`address`: *any*) => *Promise*<NativeBalance\> ; `getPrivateKey`: () => *Uint8Array* ; `getPublicKey`: () => *PublicKey* ; `getWallet`: () => *Wallet* ; `initiateChannelClosure`: (`counterparty`: *any*) => *Promise*<string\> ; `openChannel`: (`me`: *any*, `counterparty`: *any*, `amount`: *any*) => *Promise*<string\> ; `redeemTicket`: (`counterparty`: *any*, `ackTicket`: *any*, `ticket`: *any*) => *Promise*<string\> ; `setCommitment`: (`comm`: *Hash*) => *Promise*<string\> ; `subscribeBlock`: (`cb`: *any*) => *JsonRpcProvider* \| *WebSocketProvider* ; `subscribeChannelEvents`: (`cb`: *any*) => [*HoprChannels*](contracts_hoprchannels.hoprchannels.md) ; `subscribeError`: (`cb`: *any*) => *void* ; `unsubscribe`: () => *void* ; `waitUntilReady`: () => *Promise*<Network\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: *string*, `amount`: *string*) => *Promise*<string\>  }, `maxConfirmations`: *number*, `blockRange`: *number*): [*default*](indexer.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `genesisBlock` | *number* |
| `db` | *HoprDB* |
| `chain` | *object* |
| `chain.announce` | (`multiaddr`: Multiaddr) => *Promise*<string\> |
| `chain.finalizeChannelClosure` | (`counterparty`: *any*) => *Promise*<string\> |
| `chain.fundChannel` | (`me`: *Address*, `counterparty`: *Address*, `myTotal`: *Balance*, `theirTotal`: *Balance*) => *Promise*<string\> |
| `chain.getBalance` | (`address`: *Address*) => *Promise*<Balance\> |
| `chain.getChannels` | () => [*HoprChannels*](contracts_hoprchannels.hoprchannels.md) |
| `chain.getGenesisBlock` | () => *number* |
| `chain.getInfo` | () => *string* |
| `chain.getLatestBlockNumber` | () => *Promise*<number\> |
| `chain.getNativeBalance` | (`address`: *any*) => *Promise*<NativeBalance\> |
| `chain.getPrivateKey` | () => *Uint8Array* |
| `chain.getPublicKey` | () => *PublicKey* |
| `chain.getWallet` | () => *Wallet* |
| `chain.initiateChannelClosure` | (`counterparty`: *any*) => *Promise*<string\> |
| `chain.openChannel` | (`me`: *any*, `counterparty`: *any*, `amount`: *any*) => *Promise*<string\> |
| `chain.redeemTicket` | (`counterparty`: *any*, `ackTicket`: *any*, `ticket`: *any*) => *Promise*<string\> |
| `chain.setCommitment` | (`comm`: *Hash*) => *Promise*<string\> |
| `chain.subscribeBlock` | (`cb`: *any*) => *JsonRpcProvider* \| *WebSocketProvider* |
| `chain.subscribeChannelEvents` | (`cb`: *any*) => [*HoprChannels*](contracts_hoprchannels.hoprchannels.md) |
| `chain.subscribeError` | (`cb`: *any*) => *void* |
| `chain.unsubscribe` | () => *void* |
| `chain.waitUntilReady` | () => *Promise*<Network\> |
| `chain.withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: *string*, `amount`: *string*) => *Promise*<string\> |
| `maxConfirmations` | *number* |
| `blockRange` | *number* |

**Returns:** [*default*](indexer.default.md)

Overrides: EventEmitter.constructor

Defined in: [packages/core-ethereum/src/indexer/index.ts:27](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L27)

## Properties

### latestBlock

• **latestBlock**: *number*= 0

Defined in: [packages/core-ethereum/src/indexer/index.ts:26](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L26)

___

### status

• **status**: ``"started"`` \| ``"restarting"`` \| ``"stopped"``= 'stopped'

Defined in: [packages/core-ethereum/src/indexer/index.ts:25](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L25)

___

### unconfirmedEvents

• `Private` **unconfirmedEvents**: *Heap*<[*Event*](../modules/indexer_types.md#event)<any\>\>

Defined in: [packages/core-ethereum/src/indexer/index.ts:27](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L27)

___

### captureRejectionSymbol

▪ `Static` `Readonly` **captureRejectionSymbol**: *typeof* [*captureRejectionSymbol*](indexer.default.md#capturerejectionsymbol)

Inherited from: EventEmitter.captureRejectionSymbol

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:43

___

### captureRejections

▪ `Static` **captureRejections**: *boolean*

Sets or gets the default captureRejection value for all emitters.

Inherited from: EventEmitter.captureRejections

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:49

___

### defaultMaxListeners

▪ `Static` **defaultMaxListeners**: *number*

Inherited from: EventEmitter.defaultMaxListeners

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:50

___

### errorMonitor

▪ `Static` `Readonly` **errorMonitor**: *typeof* [*errorMonitor*](indexer.default.md#errormonitor)

This symbol shall be used to install a listener for only monitoring `'error'`
events. Listeners installed using this symbol are called before the regular
`'error'` listeners are called.

Installing a listener using this symbol does not change the behavior once an
`'error'` event is emitted, therefore the process will still crash if no
regular `'error'` listener is installed.

Inherited from: EventEmitter.errorMonitor

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:42

## Methods

### addListener

▸ **addListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](indexer.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](indexer.default.md)

Inherited from: EventEmitter.addListener

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:62

___

### emit

▸ **emit**(`event`: *string* \| *symbol*, ...`args`: *any*[]): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `...args` | *any*[] |

**Returns:** *boolean*

Inherited from: EventEmitter.emit

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:72

___

### eventNames

▸ **eventNames**(): (*string* \| *symbol*)[]

**Returns:** (*string* \| *symbol*)[]

Inherited from: EventEmitter.eventNames

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:77

___

### getAccount

▸ **getAccount**(`address`: *Address*): *Promise*<AccountEntry\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | *Address* |

**Returns:** *Promise*<AccountEntry\>

Defined in: [packages/core-ethereum/src/indexer/index.ts:275](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L275)

___

### getAnnouncedAddresses

▸ **getAnnouncedAddresses**(): *Promise*<Multiaddr[]\>

**Returns:** *Promise*<Multiaddr[]\>

Defined in: [packages/core-ethereum/src/indexer/index.ts:318](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L318)

___

### getChannel

▸ **getChannel**(`channelId`: *Hash*): *Promise*<ChannelEntry\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | *Hash* |

**Returns:** *Promise*<ChannelEntry\>

Defined in: [packages/core-ethereum/src/indexer/index.ts:279](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L279)

___

### getChannels

▸ **getChannels**(`filter?`: (`channel`: *ChannelEntry*) => *boolean*): *Promise*<ChannelEntry[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`channel`: *ChannelEntry*) => *boolean* |

**Returns:** *Promise*<ChannelEntry[]\>

Defined in: [packages/core-ethereum/src/indexer/index.ts:283](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L283)

___

### getChannelsFromPeer

▸ **getChannelsFromPeer**(`source`: *PeerId*): *Promise*<[*RoutingChannel*](../modules/indexer.md#routingchannel)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `source` | *PeerId* |

**Returns:** *Promise*<[*RoutingChannel*](../modules/indexer.md#routingchannel)[]\>

Defined in: [packages/core-ethereum/src/indexer/index.ts:342](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L342)

___

### getChannelsOf

▸ **getChannelsOf**(`address`: *Address*): *Promise*<ChannelEntry[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | *Address* |

**Returns:** *Promise*<ChannelEntry[]\>

Defined in: [packages/core-ethereum/src/indexer/index.ts:287](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L287)

___

### getMaxListeners

▸ **getMaxListeners**(): *number*

**Returns:** *number*

Inherited from: EventEmitter.getMaxListeners

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:69

___

### getPublicKeyOf

▸ **getPublicKeyOf**(`address`: *Address*): *Promise*<PublicKey\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | *Address* |

**Returns:** *Promise*<PublicKey\>

Defined in: [packages/core-ethereum/src/indexer/index.ts:294](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L294)

___

### getPublicNodes

▸ **getPublicNodes**(): *Promise*<Multiaddr[]\>

**Returns:** *Promise*<Multiaddr[]\>

Defined in: [packages/core-ethereum/src/indexer/index.ts:322](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L322)

___

### getRandomChannel

▸ **getRandomChannel**(): *Promise*<[*RoutingChannel*](../modules/indexer.md#routingchannel)\>

**Returns:** *Promise*<[*RoutingChannel*](../modules/indexer.md#routingchannel)\>

Defined in: [packages/core-ethereum/src/indexer/index.ts:328](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L328)

___

### listenerCount

▸ **listenerCount**(`event`: *string* \| *symbol*): *number*

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |

**Returns:** *number*

Inherited from: EventEmitter.listenerCount

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:73

___

### listeners

▸ **listeners**(`event`: *string* \| *symbol*): Function[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |

**Returns:** Function[]

Inherited from: EventEmitter.listeners

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:70

___

### off

▸ **off**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](indexer.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](indexer.default.md)

Inherited from: EventEmitter.off

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:66

___

### on

▸ **on**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](indexer.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](indexer.default.md)

Inherited from: EventEmitter.on

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:63

___

### onAnnouncement

▸ `Private` **onAnnouncement**(`event`: [*Event*](../modules/indexer_types.md#event)<``"Announcement"``\>, `blockNumber`: *BN*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | [*Event*](../modules/indexer_types.md#event)<``"Announcement"``\> |
| `blockNumber` | *BN* |

**Returns:** *Promise*<void\>

Defined in: [packages/core-ethereum/src/indexer/index.ts:246](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L246)

___

### onChannelUpdated

▸ `Private` **onChannelUpdated**(`event`: [*Event*](../modules/indexer_types.md#event)<``"ChannelUpdate"``\>): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | [*Event*](../modules/indexer_types.md#event)<``"ChannelUpdate"``\> |

**Returns:** *Promise*<void\>

Defined in: [packages/core-ethereum/src/indexer/index.ts:270](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L270)

___

### onNewBlock

▸ `Private` **onNewBlock**(`blockNumber`: *number*): *Promise*<void\>

Called whenever a new block found.
This will update {this.latestBlock},
and processes events which are within
confirmed blocks.

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockNumber` | *number* |

**Returns:** *Promise*<void\>

Defined in: [packages/core-ethereum/src/indexer/index.ts:184](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L184)

___

### onNewEvents

▸ `Private` **onNewEvents**(`events`: [*Event*](../modules/indexer_types.md#event)<any\>[]): *void*

Called whenever we receive new events.

#### Parameters

| Name | Type |
| :------ | :------ |
| `events` | [*Event*](../modules/indexer_types.md#event)<any\>[] |

**Returns:** *void*

Defined in: [packages/core-ethereum/src/indexer/index.ts:242](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L242)

___

### once

▸ **once**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](indexer.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](indexer.default.md)

Inherited from: EventEmitter.once

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:64

___

### prependListener

▸ **prependListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](indexer.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](indexer.default.md)

Inherited from: EventEmitter.prependListener

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:75

___

### prependOnceListener

▸ **prependOnceListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](indexer.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](indexer.default.md)

Inherited from: EventEmitter.prependOnceListener

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:76

___

### processPastEvents

▸ `Private` **processPastEvents**(`fromBlock`: *number*, `maxToBlock`: *number*, `maxBlockRange`: *number*): *Promise*<number\>

Query past events, this will loop until it gets all blocks from {toBlock} to {fromBlock}.
If we exceed response pull limit, we switch into quering smaller chunks.
TODO: optimize DB and fetch requests

#### Parameters

| Name | Type |
| :------ | :------ |
| `fromBlock` | *number* |
| `maxToBlock` | *number* |
| `maxBlockRange` | *number* |

**Returns:** *Promise*<number\>

past events and last queried block

Defined in: [packages/core-ethereum/src/indexer/index.ts:130](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L130)

___

### rawListeners

▸ **rawListeners**(`event`: *string* \| *symbol*): Function[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |

**Returns:** Function[]

Inherited from: EventEmitter.rawListeners

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:71

___

### removeAllListeners

▸ **removeAllListeners**(`event?`: *string* \| *symbol*): [*default*](indexer.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event?` | *string* \| *symbol* |

**Returns:** [*default*](indexer.default.md)

Inherited from: EventEmitter.removeAllListeners

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:67

___

### removeListener

▸ **removeListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](indexer.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](indexer.default.md)

Inherited from: EventEmitter.removeListener

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:65

___

### restart

▸ `Private` **restart**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [packages/core-ethereum/src/indexer/index.ts:105](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L105)

___

### setMaxListeners

▸ **setMaxListeners**(`n`: *number*): [*default*](indexer.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `n` | *number* |

**Returns:** [*default*](indexer.default.md)

Inherited from: EventEmitter.setMaxListeners

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:68

___

### start

▸ **start**(): *Promise*<void\>

Starts indexing.

**Returns:** *Promise*<void\>

Defined in: [packages/core-ethereum/src/indexer/index.ts:42](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L42)

___

### stop

▸ **stop**(): *Promise*<void\>

Stops indexing.

**Returns:** *Promise*<void\>

Defined in: [packages/core-ethereum/src/indexer/index.ts:95](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L95)

___

### toIndexerChannel

▸ `Private` **toIndexerChannel**(`source`: *PeerId*, `channel`: *ChannelEntry*): *Promise*<[*RoutingChannel*](../modules/indexer.md#routingchannel)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `source` | *PeerId* |
| `channel` | *ChannelEntry* |

**Returns:** *Promise*<[*RoutingChannel*](../modules/indexer.md#routingchannel)\>

Defined in: [packages/core-ethereum/src/indexer/index.ts:303](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/index.ts#L303)

___

### listenerCount

▸ `Static` **listenerCount**(`emitter`: *EventEmitter*, `event`: *string* \| *symbol*): *number*

**`deprecated`** since v4.0.0

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | *EventEmitter* |
| `event` | *string* \| *symbol* |

**Returns:** *number*

Inherited from: EventEmitter.listenerCount

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:31

___

### on

▸ `Static` **on**(`emitter`: *EventEmitter*, `event`: *string*): *AsyncIterableIterator*<any\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | *EventEmitter* |
| `event` | *string* |

**Returns:** *AsyncIterableIterator*<any\>

Inherited from: EventEmitter.on

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:28

___

### once

▸ `Static` **once**(`emitter`: *NodeEventTarget*, `event`: *string* \| *symbol*): *Promise*<any[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | *NodeEventTarget* |
| `event` | *string* \| *symbol* |

**Returns:** *Promise*<any[]\>

Inherited from: EventEmitter.once

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:26

▸ `Static` **once**(`emitter`: DOMEventTarget, `event`: *string*): *Promise*<any[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | DOMEventTarget |
| `event` | *string* |

**Returns:** *Promise*<any[]\>

Inherited from: EventEmitter.once

Defined in: packages/core-ethereum/node_modules/@types/node/events.d.ts:27
