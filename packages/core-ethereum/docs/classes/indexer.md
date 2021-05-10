[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / Indexer

# Class: Indexer

Indexes HoprChannels smart contract and stores to the DB,
all channels in the network.
Also keeps track of the latest block number.

## Hierarchy

- _EventEmitter_

  ↳ **Indexer**

## Table of contents

### Constructors

- [constructor](indexer.md#constructor)

### Properties

- [latestBlock](indexer.md#latestblock)
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
- [getChannelsFromPeer](indexer.md#getchannelsfrompeer)
- [getChannelsOf](indexer.md#getchannelsof)
- [getMaxListeners](indexer.md#getmaxlisteners)
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
- [once](indexer.md#once)
- [prependListener](indexer.md#prependlistener)
- [prependOnceListener](indexer.md#prependoncelistener)
- [processPastEvents](indexer.md#processpastevents)
- [rawListeners](indexer.md#rawlisteners)
- [removeAllListeners](indexer.md#removealllisteners)
- [removeListener](indexer.md#removelistener)
- [restart](indexer.md#restart)
- [setMaxListeners](indexer.md#setmaxlisteners)
- [start](indexer.md#start)
- [stop](indexer.md#stop)
- [toIndexerChannel](indexer.md#toindexerchannel)
- [listenerCount](indexer.md#listenercount)
- [on](indexer.md#on)
- [once](indexer.md#once)

## Constructors

### constructor

\+ **new Indexer**(`genesisBlock`: _number_, `db`: _HoprDB_, `chain`: { `announce`: (`multiaddr`: Multiaddr) => _Promise_<string\> ; `finalizeChannelClosure`: (`counterparty`: _any_) => _Promise_<string\> ; `fundChannel`: (`me`: _Address_, `counterparty`: _Address_, `myTotal`: _Balance_, `theirTotal`: _Balance_) => _Promise_<string\> ; `getBalance`: (`address`: _Address_) => _Promise_<Balance\> ; `getChannels`: () => _HoprChannels_ ; `getGenesisBlock`: () => _number_ ; `getInfo`: () => _string_ ; `getLatestBlockNumber`: () => _Promise_<number\> ; `getNativeBalance`: (`address`: _any_) => _Promise_<NativeBalance\> ; `getPrivateKey`: () => _Uint8Array_ ; `getPublicKey`: () => _PublicKey_ ; `getWallet`: () => _Wallet_ ; `initiateChannelClosure`: (`counterparty`: _any_) => _Promise_<string\> ; `openChannel`: (`me`: _any_, `counterparty`: _any_, `amount`: _any_) => _Promise_<string\> ; `redeemTicket`: (`counterparty`: _any_, `ackTicket`: _any_, `ticket`: _any_) => _Promise_<string\> ; `setCommitment`: (`comm`: _Hash_) => _Promise_<string\> ; `subscribeBlock`: (`cb`: _any_) => _JsonRpcProvider_ \| _WebSocketProvider_ ; `subscribeChannelEvents`: (`cb`: _any_) => _HoprChannels_ ; `subscribeError`: (`cb`: _any_) => _void_ ; `unsubscribe`: () => _void_ ; `waitUntilReady`: () => _Promise_<Network\> ; `withdraw`: (`currency`: `"NATIVE"` \| `"HOPR"`, `recipient`: _string_, `amount`: _string_) => _Promise_<string\> }, `maxConfirmations`: _number_, `blockRange`: _number_): [_Indexer_](indexer.md)

#### Parameters

| Name                           | Type                                                                                                              |
| :----------------------------- | :---------------------------------------------------------------------------------------------------------------- |
| `genesisBlock`                 | _number_                                                                                                          |
| `db`                           | _HoprDB_                                                                                                          |
| `chain`                        | _object_                                                                                                          |
| `chain.announce`               | (`multiaddr`: Multiaddr) => _Promise_<string\>                                                                    |
| `chain.finalizeChannelClosure` | (`counterparty`: _any_) => _Promise_<string\>                                                                     |
| `chain.fundChannel`            | (`me`: _Address_, `counterparty`: _Address_, `myTotal`: _Balance_, `theirTotal`: _Balance_) => _Promise_<string\> |
| `chain.getBalance`             | (`address`: _Address_) => _Promise_<Balance\>                                                                     |
| `chain.getChannels`            | () => _HoprChannels_                                                                                              |
| `chain.getGenesisBlock`        | () => _number_                                                                                                    |
| `chain.getInfo`                | () => _string_                                                                                                    |
| `chain.getLatestBlockNumber`   | () => _Promise_<number\>                                                                                          |
| `chain.getNativeBalance`       | (`address`: _any_) => _Promise_<NativeBalance\>                                                                   |
| `chain.getPrivateKey`          | () => _Uint8Array_                                                                                                |
| `chain.getPublicKey`           | () => _PublicKey_                                                                                                 |
| `chain.getWallet`              | () => _Wallet_                                                                                                    |
| `chain.initiateChannelClosure` | (`counterparty`: _any_) => _Promise_<string\>                                                                     |
| `chain.openChannel`            | (`me`: _any_, `counterparty`: _any_, `amount`: _any_) => _Promise_<string\>                                       |
| `chain.redeemTicket`           | (`counterparty`: _any_, `ackTicket`: _any_, `ticket`: _any_) => _Promise_<string\>                                |
| `chain.setCommitment`          | (`comm`: _Hash_) => _Promise_<string\>                                                                            |
| `chain.subscribeBlock`         | (`cb`: _any_) => _JsonRpcProvider_ \| _WebSocketProvider_                                                         |
| `chain.subscribeChannelEvents` | (`cb`: _any_) => _HoprChannels_                                                                                   |
| `chain.subscribeError`         | (`cb`: _any_) => _void_                                                                                           |
| `chain.unsubscribe`            | () => _void_                                                                                                      |
| `chain.waitUntilReady`         | () => _Promise_<Network\>                                                                                         |
| `chain.withdraw`               | (`currency`: `"NATIVE"` \| `"HOPR"`, `recipient`: _string_, `amount`: _string_) => _Promise_<string\>             |
| `maxConfirmations`             | _number_                                                                                                          |
| `blockRange`                   | _number_                                                                                                          |

**Returns:** [_Indexer_](indexer.md)

Overrides: EventEmitter.constructor

Defined in: [core-ethereum/src/indexer/index.ts:27](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L27)

## Properties

### latestBlock

• **latestBlock**: _number_= 0

Defined in: [core-ethereum/src/indexer/index.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L26)

---

### status

• **status**: `"started"` \| `"restarting"` \| `"stopped"`= 'stopped'

Defined in: [core-ethereum/src/indexer/index.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L25)

---

### unconfirmedEvents

• `Private` **unconfirmedEvents**: _Heap_<Event<any\>\>

Defined in: [core-ethereum/src/indexer/index.ts:27](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L27)

---

### captureRejectionSymbol

▪ `Static` `Readonly` **captureRejectionSymbol**: _typeof_ [_captureRejectionSymbol_](indexer.md#capturerejectionsymbol)

Inherited from: EventEmitter.captureRejectionSymbol

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:43

---

### captureRejections

▪ `Static` **captureRejections**: _boolean_

Sets or gets the default captureRejection value for all emitters.

Inherited from: EventEmitter.captureRejections

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:49

---

### defaultMaxListeners

▪ `Static` **defaultMaxListeners**: _number_

Inherited from: EventEmitter.defaultMaxListeners

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:50

---

### errorMonitor

▪ `Static` `Readonly` **errorMonitor**: _typeof_ [_errorMonitor_](indexer.md#errormonitor)

This symbol shall be used to install a listener for only monitoring `'error'`
events. Listeners installed using this symbol are called before the regular
`'error'` listeners are called.

Installing a listener using this symbol does not change the behavior once an
`'error'` event is emitted, therefore the process will still crash if no
regular `'error'` listener is installed.

Inherited from: EventEmitter.errorMonitor

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:42

## Methods

### addListener

▸ **addListener**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_Indexer_](indexer.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_Indexer_](indexer.md)

Inherited from: EventEmitter.addListener

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:62

---

### emit

▸ **emit**(`event`: _string_ \| _symbol_, ...`args`: _any_[]): _boolean_

#### Parameters

| Name      | Type                 |
| :-------- | :------------------- |
| `event`   | _string_ \| _symbol_ |
| `...args` | _any_[]              |

**Returns:** _boolean_

Inherited from: EventEmitter.emit

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:72

---

### eventNames

▸ **eventNames**(): (_string_ \| _symbol_)[]

**Returns:** (_string_ \| _symbol_)[]

Inherited from: EventEmitter.eventNames

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:77

---

### getAccount

▸ **getAccount**(`address`: _Address_): _Promise_<AccountEntry\>

#### Parameters

| Name      | Type      |
| :-------- | :-------- |
| `address` | _Address_ |

**Returns:** _Promise_<AccountEntry\>

Defined in: [core-ethereum/src/indexer/index.ts:275](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L275)

---

### getAnnouncedAddresses

▸ **getAnnouncedAddresses**(): _Promise_<Multiaddr[]\>

**Returns:** _Promise_<Multiaddr[]\>

Defined in: [core-ethereum/src/indexer/index.ts:318](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L318)

---

### getChannel

▸ **getChannel**(`channelId`: _Hash_): _Promise_<ChannelEntry\>

#### Parameters

| Name        | Type   |
| :---------- | :----- |
| `channelId` | _Hash_ |

**Returns:** _Promise_<ChannelEntry\>

Defined in: [core-ethereum/src/indexer/index.ts:279](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L279)

---

### getChannels

▸ **getChannels**(`filter?`: (`channel`: _ChannelEntry_) => _boolean_): _Promise_<ChannelEntry[]\>

#### Parameters

| Name      | Type                                     |
| :-------- | :--------------------------------------- |
| `filter?` | (`channel`: _ChannelEntry_) => _boolean_ |

**Returns:** _Promise_<ChannelEntry[]\>

Defined in: [core-ethereum/src/indexer/index.ts:283](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L283)

---

### getChannelsFromPeer

▸ **getChannelsFromPeer**(`source`: _PeerId_): _Promise_<[_RoutingChannel_](../modules.md#routingchannel)[]\>

#### Parameters

| Name     | Type     |
| :------- | :------- |
| `source` | _PeerId_ |

**Returns:** _Promise_<[_RoutingChannel_](../modules.md#routingchannel)[]\>

Defined in: [core-ethereum/src/indexer/index.ts:342](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L342)

---

### getChannelsOf

▸ **getChannelsOf**(`address`: _Address_): _Promise_<ChannelEntry[]\>

#### Parameters

| Name      | Type      |
| :-------- | :-------- |
| `address` | _Address_ |

**Returns:** _Promise_<ChannelEntry[]\>

Defined in: [core-ethereum/src/indexer/index.ts:287](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L287)

---

### getMaxListeners

▸ **getMaxListeners**(): _number_

**Returns:** _number_

Inherited from: EventEmitter.getMaxListeners

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:69

---

### getPublicKeyOf

▸ **getPublicKeyOf**(`address`: _Address_): _Promise_<PublicKey\>

#### Parameters

| Name      | Type      |
| :-------- | :-------- |
| `address` | _Address_ |

**Returns:** _Promise_<PublicKey\>

Defined in: [core-ethereum/src/indexer/index.ts:294](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L294)

---

### getPublicNodes

▸ **getPublicNodes**(): _Promise_<Multiaddr[]\>

**Returns:** _Promise_<Multiaddr[]\>

Defined in: [core-ethereum/src/indexer/index.ts:322](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L322)

---

### getRandomChannel

▸ **getRandomChannel**(): _Promise_<[_RoutingChannel_](../modules.md#routingchannel)\>

**Returns:** _Promise_<[_RoutingChannel_](../modules.md#routingchannel)\>

Defined in: [core-ethereum/src/indexer/index.ts:328](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L328)

---

### listenerCount

▸ **listenerCount**(`event`: _string_ \| _symbol_): _number_

#### Parameters

| Name    | Type                 |
| :------ | :------------------- |
| `event` | _string_ \| _symbol_ |

**Returns:** _number_

Inherited from: EventEmitter.listenerCount

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:73

---

### listeners

▸ **listeners**(`event`: _string_ \| _symbol_): Function[]

#### Parameters

| Name    | Type                 |
| :------ | :------------------- |
| `event` | _string_ \| _symbol_ |

**Returns:** Function[]

Inherited from: EventEmitter.listeners

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:70

---

### off

▸ **off**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_Indexer_](indexer.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_Indexer_](indexer.md)

Inherited from: EventEmitter.off

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:66

---

### on

▸ **on**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_Indexer_](indexer.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_Indexer_](indexer.md)

Inherited from: EventEmitter.on

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:63

---

### onAnnouncement

▸ `Private` **onAnnouncement**(`event`: _Event_<`"Announcement"`\>, `blockNumber`: _BN_): _Promise_<void\>

#### Parameters

| Name          | Type                       |
| :------------ | :------------------------- |
| `event`       | _Event_<`"Announcement"`\> |
| `blockNumber` | _BN_                       |

**Returns:** _Promise_<void\>

Defined in: [core-ethereum/src/indexer/index.ts:246](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L246)

---

### onChannelUpdated

▸ `Private` **onChannelUpdated**(`event`: _Event_<`"ChannelUpdate"`\>): _Promise_<void\>

#### Parameters

| Name    | Type                        |
| :------ | :-------------------------- |
| `event` | _Event_<`"ChannelUpdate"`\> |

**Returns:** _Promise_<void\>

Defined in: [core-ethereum/src/indexer/index.ts:270](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L270)

---

### onNewBlock

▸ `Private` **onNewBlock**(`blockNumber`: _number_): _Promise_<void\>

Called whenever a new block found.
This will update {this.latestBlock},
and processes events which are within
confirmed blocks.

#### Parameters

| Name          | Type     |
| :------------ | :------- |
| `blockNumber` | _number_ |

**Returns:** _Promise_<void\>

Defined in: [core-ethereum/src/indexer/index.ts:184](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L184)

---

### onNewEvents

▸ `Private` **onNewEvents**(`events`: _Event_<any\>[]): _void_

Called whenever we receive new events.

#### Parameters

| Name     | Type            |
| :------- | :-------------- |
| `events` | _Event_<any\>[] |

**Returns:** _void_

Defined in: [core-ethereum/src/indexer/index.ts:242](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L242)

---

### once

▸ **once**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_Indexer_](indexer.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_Indexer_](indexer.md)

Inherited from: EventEmitter.once

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:64

---

### prependListener

▸ **prependListener**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_Indexer_](indexer.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_Indexer_](indexer.md)

Inherited from: EventEmitter.prependListener

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:75

---

### prependOnceListener

▸ **prependOnceListener**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_Indexer_](indexer.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_Indexer_](indexer.md)

Inherited from: EventEmitter.prependOnceListener

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:76

---

### processPastEvents

▸ `Private` **processPastEvents**(`fromBlock`: _number_, `maxToBlock`: _number_, `maxBlockRange`: _number_): _Promise_<number\>

Query past events, this will loop until it gets all blocks from {toBlock} to {fromBlock}.
If we exceed response pull limit, we switch into quering smaller chunks.
TODO: optimize DB and fetch requests

#### Parameters

| Name            | Type     |
| :-------------- | :------- |
| `fromBlock`     | _number_ |
| `maxToBlock`    | _number_ |
| `maxBlockRange` | _number_ |

**Returns:** _Promise_<number\>

past events and last queried block

Defined in: [core-ethereum/src/indexer/index.ts:130](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L130)

---

### rawListeners

▸ **rawListeners**(`event`: _string_ \| _symbol_): Function[]

#### Parameters

| Name    | Type                 |
| :------ | :------------------- |
| `event` | _string_ \| _symbol_ |

**Returns:** Function[]

Inherited from: EventEmitter.rawListeners

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:71

---

### removeAllListeners

▸ **removeAllListeners**(`event?`: _string_ \| _symbol_): [_Indexer_](indexer.md)

#### Parameters

| Name     | Type                 |
| :------- | :------------------- |
| `event?` | _string_ \| _symbol_ |

**Returns:** [_Indexer_](indexer.md)

Inherited from: EventEmitter.removeAllListeners

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:67

---

### removeListener

▸ **removeListener**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_Indexer_](indexer.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_Indexer_](indexer.md)

Inherited from: EventEmitter.removeListener

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:65

---

### restart

▸ `Private` **restart**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: [core-ethereum/src/indexer/index.ts:105](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L105)

---

### setMaxListeners

▸ **setMaxListeners**(`n`: _number_): [_Indexer_](indexer.md)

#### Parameters

| Name | Type     |
| :--- | :------- |
| `n`  | _number_ |

**Returns:** [_Indexer_](indexer.md)

Inherited from: EventEmitter.setMaxListeners

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:68

---

### start

▸ **start**(): _Promise_<void\>

Starts indexing.

**Returns:** _Promise_<void\>

Defined in: [core-ethereum/src/indexer/index.ts:42](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L42)

---

### stop

▸ **stop**(): _Promise_<void\>

Stops indexing.

**Returns:** _Promise_<void\>

Defined in: [core-ethereum/src/indexer/index.ts:95](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L95)

---

### toIndexerChannel

▸ `Private` **toIndexerChannel**(`source`: _PeerId_, `channel`: _ChannelEntry_): _Promise_<[_RoutingChannel_](../modules.md#routingchannel)\>

#### Parameters

| Name      | Type           |
| :-------- | :------------- |
| `source`  | _PeerId_       |
| `channel` | _ChannelEntry_ |

**Returns:** _Promise_<[_RoutingChannel_](../modules.md#routingchannel)\>

Defined in: [core-ethereum/src/indexer/index.ts:303](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/indexer/index.ts#L303)

---

### listenerCount

▸ `Static` **listenerCount**(`emitter`: _EventEmitter_, `event`: _string_ \| _symbol_): _number_

**`deprecated`** since v4.0.0

#### Parameters

| Name      | Type                 |
| :-------- | :------------------- |
| `emitter` | _EventEmitter_       |
| `event`   | _string_ \| _symbol_ |

**Returns:** _number_

Inherited from: EventEmitter.listenerCount

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:31

---

### on

▸ `Static` **on**(`emitter`: _EventEmitter_, `event`: _string_): _AsyncIterableIterator_<any\>

#### Parameters

| Name      | Type           |
| :-------- | :------------- |
| `emitter` | _EventEmitter_ |
| `event`   | _string_       |

**Returns:** _AsyncIterableIterator_<any\>

Inherited from: EventEmitter.on

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:28

---

### once

▸ `Static` **once**(`emitter`: _NodeEventTarget_, `event`: _string_ \| _symbol_): _Promise_<any[]\>

#### Parameters

| Name      | Type                 |
| :-------- | :------------------- |
| `emitter` | _NodeEventTarget_    |
| `event`   | _string_ \| _symbol_ |

**Returns:** _Promise_<any[]\>

Inherited from: EventEmitter.once

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:26

▸ `Static` **once**(`emitter`: DOMEventTarget, `event`: _string_): _Promise_<any[]\>

#### Parameters

| Name      | Type           |
| :-------- | :------------- |
| `emitter` | DOMEventTarget |
| `event`   | _string_       |

**Returns:** _Promise_<any[]\>

Inherited from: EventEmitter.once

Defined in: core-ethereum/node_modules/@types/node/events.d.ts:27
