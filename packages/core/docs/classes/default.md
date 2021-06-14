[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / default

# Class: default

## Hierarchy

- `EventEmitter`

  ↳ **default**

## Table of contents

### Constructors

- [constructor](default.md#constructor)

### Properties

- [addressSorter](default.md#addresssorter)
- [checkTimeout](default.md#checktimeout)
- [db](default.md#db)
- [forward](default.md#forward)
- [heartbeat](default.md#heartbeat)
- [libp2p](default.md#libp2p)
- [networkPeers](default.md#networkpeers)
- [paymentChannels](default.md#paymentchannels)
- [status](default.md#status)
- [strategy](default.md#strategy)
- [captureRejectionSymbol](default.md#capturerejectionsymbol)
- [captureRejections](default.md#capturerejections)
- [defaultMaxListeners](default.md#defaultmaxlisteners)
- [errorMonitor](default.md#errormonitor)

### Methods

- [addListener](default.md#addlistener)
- [announce](default.md#announce)
- [checkBalances](default.md#checkbalances)
- [closeChannel](default.md#closechannel)
- [connectionReport](default.md#connectionreport)
- [emit](default.md#emit)
- [eventNames](default.md#eventnames)
- [fundChannel](default.md#fundchannel)
- [getAcknowledgedTickets](default.md#getacknowledgedtickets)
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
- [getOpenChannels](default.md#getopenchannels)
- [getPublicKeyOf](default.md#getpublickeyof)
- [getTicketStatistics](default.md#getticketstatistics)
- [getVersion](default.md#getversion)
- [listenerCount](default.md#listenercount)
- [listeners](default.md#listeners)
- [maybeLogProfilingToGCloud](default.md#maybelogprofilingtogcloud)
- [off](default.md#off)
- [on](default.md#on)
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
| `options` | [HoprOptions](../modules.md#hoproptions) |

#### Overrides

EventEmitter.constructor

#### Defined in

[packages/core/src/index.ts:104](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L104)

## Properties

### addressSorter

• `Private` **addressSorter**: `AddressSorter`

#### Defined in

[packages/core/src/index.ts:104](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L104)

___

### checkTimeout

• `Private` **checkTimeout**: `Timeout`

#### Defined in

[packages/core/src/index.ts:96](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L96)

___

### db

• `Private` **db**: `HoprDB`

#### Defined in

[packages/core/src/index.ts:102](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L102)

___

### forward

• `Private` **forward**: `PacketForwardInteraction`

#### Defined in

[packages/core/src/index.ts:100](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L100)

___

### heartbeat

• `Private` **heartbeat**: `default`

#### Defined in

[packages/core/src/index.ts:99](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L99)

___

### libp2p

• `Private` **libp2p**: [LibP2P](libp2p.md)

#### Defined in

[packages/core/src/index.ts:101](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L101)

___

### networkPeers

• `Private` **networkPeers**: `NetworkPeers`

#### Defined in

[packages/core/src/index.ts:98](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L98)

___

### paymentChannels

• `Private` **paymentChannels**: `Promise`<default\>

#### Defined in

[packages/core/src/index.ts:103](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L103)

___

### status

• **status**: [NodeStatus](../modules.md#nodestatus) = 'UNINITIALIZED'

#### Defined in

[packages/core/src/index.ts:94](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L94)

___

### strategy

• `Private` **strategy**: `ChannelStrategy`

#### Defined in

[packages/core/src/index.ts:97](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L97)

___

### captureRejectionSymbol

▪ `Static` `Readonly` **captureRejectionSymbol**: typeof [captureRejectionSymbol](default.md#capturerejectionsymbol)

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

▪ `Static` `Readonly` **errorMonitor**: typeof [errorMonitor](default.md#errormonitor)

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

▸ **addListener**(`event`, `listener`): [default](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[default](default.md)

#### Inherited from

EventEmitter.addListener

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:62

___

### announce

▸ `Private` **announce**(`includeRouting?`): `Promise`<void\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `includeRouting` | `boolean` | false |

#### Returns

`Promise`<void\>

#### Defined in

[packages/core/src/index.ts:570](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L570)

___

### checkBalances

▸ `Private` **checkBalances**(): `Promise`<void\>

#### Returns

`Promise`<void\>

#### Defined in

[packages/core/src/index.ts:542](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L542)

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

[packages/core/src/index.ts:703](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L703)

___

### connectionReport

▸ **connectionReport**(): `Promise`<string\>

#### Returns

`Promise`<string\>

#### Defined in

[packages/core/src/index.ts:531](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L531)

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

[packages/core/src/index.ts:675](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L675)

___

### getAcknowledgedTickets

▸ **getAcknowledgedTickets**(): `Promise`<AcknowledgedTicket[]\>

#### Returns

`Promise`<AcknowledgedTicket[]\>

#### Defined in

[packages/core/src/index.ts:726](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L726)

___

### getAnnouncedAddresses

▸ **getAnnouncedAddresses**(`peer?`): `Promise`<Multiaddr[]\>

Lists the addresses which the given node announces to other nodes

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | `PeerId` | peer to query for, default self |

#### Returns

`Promise`<Multiaddr[]\>

#### Defined in

[packages/core/src/index.ts:372](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L372)

___

### getBalance

▸ **getBalance**(): `Promise`<Balance\>

#### Returns

`Promise`<Balance\>

#### Defined in

[packages/core/src/index.ts:618](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L618)

___

### getChannelStrategy

▸ **getChannelStrategy**(): `string`

#### Returns

`string`

#### Defined in

[packages/core/src/index.ts:614](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L614)

___

### getChannelsFrom

▸ **getChannelsFrom**(`addr`): `Promise`<ChannelEntry[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<ChannelEntry[]\>

#### Defined in

[packages/core/src/index.ts:778](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L778)

___

### getChannelsTo

▸ **getChannelsTo**(`addr`): `Promise`<ChannelEntry[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<ChannelEntry[]\>

#### Defined in

[packages/core/src/index.ts:783](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L783)

___

### getConnectedPeers

▸ **getConnectedPeers**(): `PeerId`[]

#### Returns

`PeerId`[]

#### Defined in

[packages/core/src/index.ts:524](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L524)

___

### getEthereumAddress

▸ **getEthereumAddress**(): `Promise`<Address\>

#### Returns

`Promise`<Address\>

#### Defined in

[packages/core/src/index.ts:793](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L793)

___

### getId

▸ **getId**(): `PeerId`

#### Returns

`PeerId`

#### Defined in

[packages/core/src/index.ts:364](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L364)

___

### getIntermediateNodes

▸ `Private` **getIntermediateNodes**(`destination`): `Promise`<PeerId[]\>

Takes a destination and samples randomly intermediate nodes
that will relay that message before it reaches its destination.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `destination` | `PeerId` | instance of peerInfo that contains the peerId of the destination |

#### Returns

`Promise`<PeerId[]\>

#### Defined in

[packages/core/src/index.ts:809](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L809)

___

### getListeningAddresses

▸ **getListeningAddresses**(): `Multiaddr`[]

List the addresses on which the node is listening

#### Returns

`Multiaddr`[]

#### Defined in

[packages/core/src/index.ts:394](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L394)

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

▸ **getNativeBalance**(): `Promise`<NativeBalance\>

#### Returns

`Promise`<NativeBalance\>

#### Defined in

[packages/core/src/index.ts:623](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L623)

___

### getObservedAddresses

▸ **getObservedAddresses**(`peer`): `Address`[]

Gets the observed addresses of a given peer.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | `PeerId` | peer to query for |

#### Returns

`Address`[]

#### Defined in

[packages/core/src/index.ts:402](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L402)

___

### getOpenChannels

▸ `Private` **getOpenChannels**(): `Promise`<RoutingChannel[]\>

#### Returns

`Promise`<RoutingChannel[]\>

#### Defined in

[packages/core/src/index.ts:339](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L339)

___

### getPublicKeyOf

▸ **getPublicKeyOf**(`addr`): `Promise`<PublicKey\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | `Address` |

#### Returns

`Promise`<PublicKey\>

#### Defined in

[packages/core/src/index.ts:788](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L788)

___

### getTicketStatistics

▸ **getTicketStatistics**(): `Promise`<`Object`\>

#### Returns

`Promise`<`Object`\>

#### Defined in

[packages/core/src/index.ts:730](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L730)

___

### getVersion

▸ **getVersion**(): `any`

Returns the version of hopr-core.

#### Returns

`any`

#### Defined in

[packages/core/src/index.ts:346](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L346)

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

[packages/core/src/index.ts:278](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L278)

___

### off

▸ **off**(`event`, `listener`): [default](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[default](default.md)

#### Inherited from

EventEmitter.off

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:66

___

### on

▸ **on**(`event`, `listener`): [default](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[default](default.md)

#### Inherited from

EventEmitter.on

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:63

___

### once

▸ **once**(`event`, `listener`): [default](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[default](default.md)

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

[packages/core/src/index.ts:644](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L644)

___

### periodicCheck

▸ `Private` **periodicCheck**(): `Promise`<void\>

#### Returns

`Promise`<void\>

#### Defined in

[packages/core/src/index.ts:556](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L556)

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

[packages/core/src/index.ts:506](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L506)

___

### prependListener

▸ **prependListener**(`event`, `listener`): [default](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[default](default.md)

#### Inherited from

EventEmitter.prependListener

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:75

___

### prependOnceListener

▸ **prependOnceListener**(`event`, `listener`): [default](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[default](default.md)

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

▸ **redeemAcknowledgedTicket**(`ackTicket`): `Promise`<RedeemTicketResponse\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackTicket` | `AcknowledgedTicket` |

#### Returns

`Promise`<RedeemTicketResponse\>

#### Defined in

[packages/core/src/index.ts:772](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L772)

___

### redeemAllTickets

▸ **redeemAllTickets**(): `Promise`<`Object`\>

#### Returns

`Promise`<`Object`\>

#### Defined in

[packages/core/src/index.ts:748](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L748)

___

### removeAllListeners

▸ **removeAllListeners**(`event?`): [default](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event?` | `string` \| `symbol` |

#### Returns

[default](default.md)

#### Inherited from

EventEmitter.removeAllListeners

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:67

___

### removeListener

▸ **removeListener**(`event`, `listener`): [default](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | `string` \| `symbol` |
| `listener` | (...`args`: `any`[]) => `void` |

#### Returns

[default](default.md)

#### Inherited from

EventEmitter.removeListener

#### Defined in

packages/core/node_modules/@types/node/events.d.ts:65

___

### sendMessage

▸ **sendMessage**(`msg`, `destination`, `intermediatePath?`): `Promise`<void\>

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `msg` | `Uint8Array` | message to send |
| `destination` | `PeerId` | PeerId of the destination |
| `intermediatePath?` | `PeerId`[] | - |

#### Returns

`Promise`<void\>

#### Defined in

[packages/core/src/index.ts:411](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L411)

___

### setChannelStrategy

▸ **setChannelStrategy**(`strategy`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `strategy` | [ChannelStrategyNames](../modules.md#channelstrategynames) |

#### Returns

`void`

#### Defined in

[packages/core/src/index.ts:602](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L602)

___

### setMaxListeners

▸ **setMaxListeners**(`n`): [default](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `n` | `number` |

#### Returns

[default](default.md)

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

[packages/core/src/index.ts:628](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L628)

___

### start

▸ **start**(): `Promise`<void\>

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

`Promise`<void\>

#### Defined in

[packages/core/src/index.ts:162](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L162)

___

### stop

▸ **stop**(): `Promise`<void\>

Shuts down the node and saves keys and peerBook in the database

#### Returns

`Promise`<void\>

#### Defined in

[packages/core/src/index.ts:353](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L353)

___

### tickChannelStrategy

▸ `Private` **tickChannelStrategy**(`newChannels`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `newChannels` | `RoutingChannel`[] |

#### Returns

`Promise`<void\>

#### Defined in

[packages/core/src/index.ts:297](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L297)

___

### waitForFunds

▸ **waitForFunds**(): `Promise`<void\>

#### Returns

`Promise`<void\>

#### Defined in

[packages/core/src/index.ts:822](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L822)

___

### waitForRunning

▸ **waitForRunning**(): `Promise`<void\>

#### Returns

`Promise`<void\>

#### Defined in

[packages/core/src/index.ts:839](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L839)

___

### withdraw

▸ **withdraw**(`currency`, `recipient`, `amount`): `Promise`<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `currency` | ``"NATIVE"`` \| ``"HOPR"`` |
| `recipient` | `string` |
| `amount` | `string` |

#### Returns

`Promise`<string\>

#### Defined in

[packages/core/src/index.ts:798](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L798)

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

packages/core/node_modules/@types/node/events.d.ts:28

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

packages/core/node_modules/@types/node/events.d.ts:26

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

packages/core/node_modules/@types/node/events.d.ts:27
