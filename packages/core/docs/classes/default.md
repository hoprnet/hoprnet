[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / default

# Class: default

## Hierarchy

- `EventEmitter`

  ↳ **`default`**

## Table of contents

### Constructors

- [constructor](default.md#constructor)

### Properties

- [addressSorter](default.md#addresssorter)
- [checkTimeout](default.md#checktimeout)
- [db](default.md#db)
- [forward](default.md#forward)
- [heartbeat](default.md#heartbeat)
- [indexer](default.md#indexer)
- [libp2p](default.md#libp2p)
- [networkPeers](default.md#networkpeers)
- [paymentChannels](default.md#paymentchannels)
- [publicNodesEmitter](default.md#publicnodesemitter)
- [status](default.md#status)
- [strategy](default.md#strategy)
- [captureRejectionSymbol](default.md#capturerejectionsymbol)
- [captureRejections](default.md#capturerejections)
- [defaultMaxListeners](default.md#defaultmaxlisteners)
- [errorMonitor](default.md#errormonitor)

### Methods

- [addListener](default.md#addlistener)
- [addPreviousNodes](default.md#addpreviousnodes)
- [announce](default.md#announce)
- [closeChannel](default.md#closechannel)
- [connectionReport](default.md#connectionreport)
- [emit](default.md#emit)
- [eventNames](default.md#eventnames)
- [fundChannel](default.md#fundchannel)
- [getAcknowledgedTickets](default.md#getacknowledgedtickets)
- [getAllChannels](default.md#getallchannels)
- [getAnnouncedAddresses](default.md#getannouncedaddresses)
- [getBalance](default.md#getbalance)
- [getChannelStrategy](default.md#getchannelstrategy)
- [getChannelsFrom](default.md#getchannelsfrom)
- [getChannelsTo](default.md#getchannelsto)
- [getConnectedPeers](default.md#getconnectedpeers)
- [getEthereumAddress](default.md#getethereumaddress)
- [getId](default.md#getid)
- [getIntermediateNodes](default.md#getintermediatenodes)
- [getListeningAddresses](default.md#getlisteningaddresses)
- [getMaxListeners](default.md#getmaxlisteners)
- [getNativeBalance](default.md#getnativebalance)
- [getObservedAddresses](default.md#getobservedaddresses)
- [getPublicKeyOf](default.md#getpublickeyof)
- [getTicketStatistics](default.md#getticketstatistics)
- [getVersion](default.md#getversion)
- [isOutOfFunds](default.md#isoutoffunds)
- [listenerCount](default.md#listenercount)
- [listeners](default.md#listeners)
- [maybeLogProfilingToGCloud](default.md#maybelogprofilingtogcloud)
- [off](default.md#off)
- [on](default.md#on)
- [onPeerAnnouncement](default.md#onpeerannouncement)
- [once](default.md#once)
- [openChannel](default.md#openchannel)
- [periodicCheck](default.md#periodiccheck)
- [ping](default.md#ping)
- [prependListener](default.md#prependlistener)
- [prependOnceListener](default.md#prependoncelistener)
- [rawListeners](default.md#rawlisteners)
- [redeemAcknowledgedTicket](default.md#redeemacknowledgedticket)
- [redeemAllTickets](default.md#redeemalltickets)
- [removeAllListeners](default.md#removealllisteners)
- [removeListener](default.md#removelistener)
- [sendMessage](default.md#sendmessage)
- [setChannelStrategy](default.md#setchannelstrategy)
- [setMaxListeners](default.md#setmaxlisteners)
- [smartContractInfo](default.md#smartcontractinfo)
- [start](default.md#start)
- [startedPaymentChannels](default.md#startedpaymentchannels)
- [stop](default.md#stop)
- [tickChannelStrategy](default.md#tickchannelstrategy)
- [waitForFunds](default.md#waitforfunds)
- [waitForRunning](default.md#waitforrunning)
- [withdraw](default.md#withdraw)
- [listenerCount](default.md#listenercount)
- [on](default.md#on)
- [once](default.md#once)

## Constructors

### constructor

• **new default**(`id`, `options`)

Create an uninitialized Hopr Node

#### Parameters

| Name | Type |
| :------ | :------ |
| `id` | `PeerId` |
| `options` | [`HoprOptions`](../modules.md#hoproptions) |

#### Overrides

EventEmitter.constructor

#### Defined in

[packages/core/src/index.ts:116](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L116)

## Properties

### addressSorter

• `Private` **addressSorter**: `AddressSorter`

#### Defined in

[packages/core/src/index.ts:103](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L103)

___

### checkTimeout

• `Private` **checkTimeout**: `Timeout`

#### Defined in

[packages/core/src/index.ts:95](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L95)

___

### db

• `Private` **db**: `HoprDB`

#### Defined in

[packages/core/src/index.ts:101](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L101)

___

### forward

• `Private` **forward**: `PacketForwardInteraction`

#### Defined in

[packages/core/src/index.ts:99](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L99)

___

### heartbeat

• `Private` **heartbeat**: `default`

#### Defined in

[packages/core/src/index.ts:98](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L98)

___

### indexer

• **indexer**: `Indexer`

#### Defined in

[packages/core/src/index.ts:106](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L106)

___

### libp2p

• `Private` **libp2p**: [`LibP2P`](LibP2P.md)

#### Defined in

[packages/core/src/index.ts:100](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L100)

___

### networkPeers

• `Private` **networkPeers**: `NetworkPeers`

#### Defined in

[packages/core/src/index.ts:97](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L97)

___

### paymentChannels

• `Private` **paymentChannels**: `default`

#### Defined in

[packages/core/src/index.ts:102](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L102)

___

### publicNodesEmitter

• `Private` **publicNodesEmitter**: `PublicNodesEmitter`

#### Defined in

[packages/core/src/index.ts:104](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L104)

___

### status

• **status**: [`NodeStatus`](../modules.md#nodestatus) = `'UNINITIALIZED'`

#### Defined in

[packages/core/src/index.ts:93](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L93)

___

### strategy

• `Private` **strategy**: `ChannelStrategy`

#### Defined in

[packages/core/src/index.ts:96](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L96)

___

### captureRejectionSymbol

▪ `Static` `Readonly` **captureRejectionSymbol**: typeof [`captureRejectionSymbol`](default.md#capturerejectionsymbol)

#### Inherited from

EventEmitter.captureRejectionSymbol

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:43

___

### captureRejections

▪ `Static` **captureRejections**: `boolean`

Sets or gets the default captureRejection value for all emitters.

#### Inherited from

EventEmitter.captureRejections

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:49

___

### defaultMaxListeners

▪ `Static` **defaultMaxListeners**: `number`

#### Inherited from

EventEmitter.defaultMaxListeners

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:50

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

packages/core/node_modules/@types/node/events.d.ts:42

## Methods

### addListener

▸ **addListener**(`event`, `listener`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.addListener

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:62

___

### addPreviousNodes

▸ `Private` **addPreviousNodes**(`initialNodes`): `void`

Populates libp2p's peerStore with previously announced nodes

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `initialNodes` | `Multiaddr`[] | previosly announced nodes |

#### Returns

`void`

#### Defined in

[packages/core/src/index.ts:320](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L320)

___

### announce

▸ `Private` **announce**(`includeRouting?`): `Promise`<`void`\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `includeRouting` | `boolean` | `false` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:632](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L632)

___

### closeChannel

▸ **closeChannel**(`counterparty`): `Promise`<`Object`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | `PeerId` |

#### Returns

`Promise`<`Object`\>

#### Defined in

[packages/core/src/index.ts:776](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L776)

___

### connectionReport

▸ **connectionReport**(): `Promise`<`string`\>

#### Returns

`Promise`<`string`\>

#### Defined in

[packages/core/src/index.ts:608](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L608)

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

packages/core/node_modules/@types/node/events.d.ts:72

___

### eventNames

▸ **eventNames**(): (`string` \| `symbol`)[]

#### Returns

(`string` \| `symbol`)[]

#### Inherited from

EventEmitter.eventNames

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:77

___

### fundChannel

▸ **fundChannel**(`counterparty`, `myFund`, `counterpartyFund`): `Promise`<`Object`\>

Fund a payment channel

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `counterparty` | `PeerId` | the counter party's peerId |
| `myFund` | `BN` | the amount to fund the channel in my favor HOPR(wei) |
| `counterpartyFund` | `BN` | the amount to fund the channel in counterparty's favor HOPR(wei) |

#### Returns

`Promise`<`Object`\>

#### Defined in

[packages/core/src/index.ts:742](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L742)

___

### getAcknowledgedTickets

▸ **getAcknowledgedTickets**(): `Promise`<`AcknowledgedTicket`[]\>

#### Returns

`Promise`<`AcknowledgedTicket`[]\>

#### Defined in

[packages/core/src/index.ts:807](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L807)

___

### getAllChannels

▸ `Private` **getAllChannels**(): `Promise`<`ChannelEntry`[]\>

#### Returns

`Promise`<`ChannelEntry`[]\>

#### Defined in

[packages/core/src/index.ts:411](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L411)

___

### getAnnouncedAddresses

▸ **getAnnouncedAddresses**(`peer?`): `Promise`<`Multiaddr`[]\>

List of addresses that is announced to other nodes

**`dev`** returned list can change at runtime

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | `PeerId` | peer to query for, default self |

#### Returns

`Promise`<`Multiaddr`[]\>

#### Defined in

[packages/core/src/index.ts:445](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L445)

___

### getBalance

▸ **getBalance**(): `Promise`<`Balance`\>

#### Returns

`Promise`<`Balance`\>

#### Defined in

[packages/core/src/index.ts:676](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L676)

___

### getChannelStrategy

▸ **getChannelStrategy**(): `string`

#### Returns

`string`

#### Defined in

[packages/core/src/index.ts:672](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L672)

___

### getChannelsFrom

▸ **getChannelsFrom**(`addr`): `Promise`<`ChannelEntry`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<`ChannelEntry`[]\>

#### Defined in

[packages/core/src/index.ts:865](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L865)

___

### getChannelsTo

▸ **getChannelsTo**(`addr`): `Promise`<`ChannelEntry`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<`ChannelEntry`[]\>

#### Defined in

[packages/core/src/index.ts:870](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L870)

___

### getConnectedPeers

▸ **getConnectedPeers**(): `PeerId`[]

#### Returns

`PeerId`[]

#### Defined in

[packages/core/src/index.ts:601](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L601)

___

### getEthereumAddress

▸ **getEthereumAddress**(): `Promise`<`Address`\>

#### Returns

`Promise`<`Address`\>

#### Defined in

[packages/core/src/index.ts:880](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L880)

___

### getId

▸ **getId**(): `PeerId`

#### Returns

`PeerId`

#### Defined in

[packages/core/src/index.ts:436](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L436)

___

### getIntermediateNodes

▸ `Private` **getIntermediateNodes**(`destination`): `Promise`<`PublicKey`[]\>

Takes a destination and samples randomly intermediate nodes
that will relay that message before it reaches its destination.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `destination` | `PublicKey` | instance of peerInfo that contains the peerId of the destination |

#### Returns

`Promise`<`PublicKey`[]\>

#### Defined in

[packages/core/src/index.ts:902](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L902)

___

### getListeningAddresses

▸ **getListeningAddresses**(): `Multiaddr`[]

List the addresses on which the node is listening

#### Returns

`Multiaddr`[]

#### Defined in

[packages/core/src/index.ts:467](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L467)

___

### getMaxListeners

▸ **getMaxListeners**(): `number`

#### Returns

`number`

#### Inherited from

EventEmitter.getMaxListeners

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:69

___

### getNativeBalance

▸ **getNativeBalance**(): `Promise`<`NativeBalance`\>

#### Returns

`Promise`<`NativeBalance`\>

#### Defined in

[packages/core/src/index.ts:681](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L681)

___

### getObservedAddresses

▸ **getObservedAddresses**(`peer`): `Multiaddr`[]

Gets the observed addresses of a given peer.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | `PeerId` | peer to query for |

#### Returns

`Multiaddr`[]

#### Defined in

[packages/core/src/index.ts:475](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L475)

___

### getPublicKeyOf

▸ **getPublicKeyOf**(`addr`): `Promise`<`PublicKey`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<`PublicKey`\>

#### Defined in

[packages/core/src/index.ts:875](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L875)

___

### getTicketStatistics

▸ **getTicketStatistics**(): `Promise`<`Object`\>

#### Returns

`Promise`<`Object`\>

#### Defined in

[packages/core/src/index.ts:811](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L811)

___

### getVersion

▸ **getVersion**(): `any`

Returns the version of hopr-core.

#### Returns

`any`

#### Defined in

[packages/core/src/index.ts:418](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L418)

___

### isOutOfFunds

▸ `Private` **isOutOfFunds**(`error`): `Promise`<`void`\>

If error provided is considered an out of funds error
- it will emit that the node is out of funds

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `error` | `any` | error thrown by an ethereum transaction |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:298](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L298)

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

packages/core/node_modules/@types/node/events.d.ts:73

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

packages/core/node_modules/@types/node/events.d.ts:70

___

### maybeLogProfilingToGCloud

▸ `Private` **maybeLogProfilingToGCloud**(): `void`

#### Returns

`void`

#### Defined in

[packages/core/src/index.ts:274](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L274)

___

### off

▸ **off**(`event`, `listener`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.off

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:66

___

### on

▸ **on**(`event`, `listener`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.on

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:63

___

### onPeerAnnouncement

▸ `Private` **onPeerAnnouncement**(`peer`): `void`

Called whenever a peer is announced

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | `Object` | newly announced peer |
| `peer.id` | `PeerId` | - |
| `peer.multiaddrs` | `Multiaddr`[] | - |

#### Returns

`void`

#### Defined in

[packages/core/src/index.ts:346](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L346)

___

### once

▸ **once**(`event`, `listener`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.once

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:64

___

### openChannel

▸ **openChannel**(`counterparty`, `amountToFund`): `Promise`<`Object`\>

Open a payment channel

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `counterparty` | `PeerId` | the counter party's peerId |
| `amountToFund` | `BN` | the amount to fund in HOPR(wei) |

#### Returns

`Promise`<`Object`\>

#### Defined in

[packages/core/src/index.ts:702](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L702)

___

### periodicCheck

▸ `Private` **periodicCheck**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:619](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L619)

___

### ping

▸ **ping**(`destination`): `Promise`<`Object`\>

Ping a node.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `destination` | `PeerId` | PeerId of the node |

#### Returns

`Promise`<`Object`\>

latency

#### Defined in

[packages/core/src/index.ts:583](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L583)

___

### prependListener

▸ **prependListener**(`event`, `listener`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.prependListener

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:75

___

### prependOnceListener

▸ **prependOnceListener**(`event`, `listener`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.prependOnceListener

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:76

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

packages/core/node_modules/@types/node/events.d.ts:71

___

### redeemAcknowledgedTicket

▸ **redeemAcknowledgedTicket**(`ackTicket`): `Promise`<`RedeemTicketResponse`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackTicket` | `AcknowledgedTicket` |

#### Returns

`Promise`<`RedeemTicketResponse`\>

#### Defined in

[packages/core/src/index.ts:853](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L853)

___

### redeemAllTickets

▸ **redeemAllTickets**(): `Promise`<`Object`\>

#### Returns

`Promise`<`Object`\>

#### Defined in

[packages/core/src/index.ts:829](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L829)

___

### removeAllListeners

▸ **removeAllListeners**(`event?`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event?` | `string` \| `symbol` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.removeAllListeners

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:67

___

### removeListener

▸ **removeListener**(`event`, `listener`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.removeListener

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:65

___

### sendMessage

▸ **sendMessage**(`msg`, `destination`, `intermediatePath?`): `Promise`<`void`\>

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `msg` | `Uint8Array` | message to send |
| `destination` | `PeerId` | PeerId of the destination |
| `intermediatePath?` | `PublicKey`[] | - |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:484](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L484)

___

### setChannelStrategy

▸ **setChannelStrategy**(`strategy`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `strategy` | `ChannelStrategy` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:663](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L663)

___

### setMaxListeners

▸ **setMaxListeners**(`n`): [`default`](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `n` | `number` |

#### Returns

[`default`](default.md)

#### Inherited from

EventEmitter.setMaxListeners

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:68

___

### smartContractInfo

▸ **smartContractInfo**(): `Promise`<`Object`\>

#### Returns

`Promise`<`Object`\>

#### Defined in

[packages/core/src/index.ts:686](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L686)

___

### start

▸ **start**(): `Promise`<`void`\>

Start node

The node has a fairly complex lifecycle. This method should do all setup
required for a node to be functioning.

If the node is not funded, it will throw.

- Create a link to the ethereum blockchain
  - Finish indexing previous blocks [SLOW]
  - Find publicly accessible relays

- Start LibP2P and work out our network configuration.
  - Pass the list of relays from the indexer

- Wait for wallet to be funded with ETH [requires user interaction]

- Announce address, pubkey, and multiaddr on chain.

- Start heartbeat, automatic strategies, etc..

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:174](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L174)

___

### startedPaymentChannels

▸ `Private` **startedPaymentChannels**(): `Promise`<`default`\>

#### Returns

`Promise`<`default`\>

#### Defined in

[packages/core/src/index.ts:147](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L147)

___

### stop

▸ **stop**(): `Promise`<`void`\>

Shuts down the node and saves keys and peerBook in the database

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:425](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L425)

___

### tickChannelStrategy

▸ `Private` **tickChannelStrategy**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:368](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L368)

___

### waitForFunds

▸ **waitForFunds**(): `Promise`<`void`\>

This is a utility method to wait until the node is funded.
A backoff is implemented, if node has not been funded and
MAX_DELAY is reached, this function will reject.

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:918](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L918)

___

### waitForRunning

▸ **waitForRunning**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/index.ts:949](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L949)

___

### withdraw

▸ **withdraw**(`currency`, `recipient`, `amount`): `Promise`<`string`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `currency` | ``"NATIVE"`` \| ``"HOPR"`` |
| `recipient` | `string` |
| `amount` | `string` |

#### Returns

`Promise`<`string`\>

#### Defined in

[packages/core/src/index.ts:885](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L885)

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

packages/core/node_modules/@types/node/events.d.ts:31

___

### on

▸ `Static` **on**(`emitter`, `event`): `AsyncIterableIterator`<`any`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `EventEmitter` |
| `event` | `string` |

#### Returns

`AsyncIterableIterator`<`any`\>

#### Inherited from

EventEmitter.on

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:28

___

### once

▸ `Static` **once**(`emitter`, `event`): `Promise`<`any`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `NodeEventTarget` |
| `event` | `string` \| `symbol` |

#### Returns

`Promise`<`any`[]\>

#### Inherited from

EventEmitter.once

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:26

▸ `Static` **once**(`emitter`, `event`): `Promise`<`any`[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | `DOMEventTarget` |
| `event` | `string` |

#### Returns

`Promise`<`any`[]\>

#### Inherited from

EventEmitter.once

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:27
