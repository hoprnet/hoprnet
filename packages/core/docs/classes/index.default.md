[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / [index](../modules/index.md) / default

# Class: default

[index](../modules/index.md).default

## Hierarchy

- *EventEmitter*

  ↳ **default**

## Table of contents

### Constructors

- [constructor](index.default.md#constructor)

### Properties

- [checkTimeout](index.default.md#checktimeout)
- [db](index.default.md#db)
- [forward](index.default.md#forward)
- [heartbeat](index.default.md#heartbeat)
- [libp2p](index.default.md#libp2p)
- [networkPeers](index.default.md#networkpeers)
- [paymentChannels](index.default.md#paymentchannels)
- [status](index.default.md#status)
- [strategy](index.default.md#strategy)
- [captureRejectionSymbol](index.default.md#capturerejectionsymbol)
- [captureRejections](index.default.md#capturerejections)
- [defaultMaxListeners](index.default.md#defaultmaxlisteners)
- [errorMonitor](index.default.md#errormonitor)

### Methods

- [addListener](index.default.md#addlistener)
- [announce](index.default.md#announce)
- [checkBalances](index.default.md#checkbalances)
- [closeChannel](index.default.md#closechannel)
- [connectionReport](index.default.md#connectionreport)
- [emit](index.default.md#emit)
- [eventNames](index.default.md#eventnames)
- [fundChannel](index.default.md#fundchannel)
- [getAcknowledgedTickets](index.default.md#getacknowledgedtickets)
- [getAnnouncedAddresses](index.default.md#getannouncedaddresses)
- [getBalance](index.default.md#getbalance)
- [getChannelStrategy](index.default.md#getchannelstrategy)
- [getChannelsOf](index.default.md#getchannelsof)
- [getConnectedPeers](index.default.md#getconnectedpeers)
- [getEthereumAddress](index.default.md#getethereumaddress)
- [getId](index.default.md#getid)
- [getIntermediateNodes](index.default.md#getintermediatenodes)
- [getListeningAddresses](index.default.md#getlisteningaddresses)
- [getMaxListeners](index.default.md#getmaxlisteners)
- [getNativeBalance](index.default.md#getnativebalance)
- [getObservedAddresses](index.default.md#getobservedaddresses)
- [getOpenChannels](index.default.md#getopenchannels)
- [getPublicKeyOf](index.default.md#getpublickeyof)
- [getVersion](index.default.md#getversion)
- [listenerCount](index.default.md#listenercount)
- [listeners](index.default.md#listeners)
- [maybeLogProfilingToGCloud](index.default.md#maybelogprofilingtogcloud)
- [off](index.default.md#off)
- [on](index.default.md#on)
- [once](index.default.md#once)
- [openChannel](index.default.md#openchannel)
- [periodicCheck](index.default.md#periodiccheck)
- [ping](index.default.md#ping)
- [prependListener](index.default.md#prependlistener)
- [prependOnceListener](index.default.md#prependoncelistener)
- [rawListeners](index.default.md#rawlisteners)
- [removeAllListeners](index.default.md#removealllisteners)
- [removeListener](index.default.md#removelistener)
- [sendMessage](index.default.md#sendmessage)
- [setChannelStrategy](index.default.md#setchannelstrategy)
- [setMaxListeners](index.default.md#setmaxlisteners)
- [smartContractInfo](index.default.md#smartcontractinfo)
- [start](index.default.md#start)
- [stop](index.default.md#stop)
- [submitAcknowledgedTicket](index.default.md#submitacknowledgedticket)
- [tickChannelStrategy](index.default.md#tickchannelstrategy)
- [waitForFunds](index.default.md#waitforfunds)
- [withdraw](index.default.md#withdraw)
- [listenerCount](index.default.md#listenercount)
- [on](index.default.md#on)
- [once](index.default.md#once)

## Constructors

### constructor

\+ **new default**(`id`: *PeerId*, `options`: [*HoprOptions*](../modules/index.md#hoproptions)): [*default*](index.default.md)

Create an uninitialized Hopr Node

#### Parameters

| Name | Type |
| :------ | :------ |
| `id` | *PeerId* |
| `options` | [*HoprOptions*](../modules/index.md#hoproptions) |

**Returns:** [*default*](index.default.md)

Overrides: EventEmitter.constructor

Defined in: [packages/core/src/index.ts:97](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L97)

## Properties

### checkTimeout

• `Private` **checkTimeout**: *Timeout*

Defined in: [packages/core/src/index.ts:90](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L90)

___

### db

• `Private` **db**: *HoprDB*

Defined in: [packages/core/src/index.ts:96](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L96)

___

### forward

• `Private` **forward**: [*PacketForwardInteraction*](interactions_packet_forward.packetforwardinteraction.md)

Defined in: [packages/core/src/index.ts:94](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L94)

___

### heartbeat

• `Private` **heartbeat**: [*default*](network_heartbeat.default.md)

Defined in: [packages/core/src/index.ts:93](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L93)

___

### libp2p

• `Private` **libp2p**: [*LibP2P*](index.libp2p-1.md)

Defined in: [packages/core/src/index.ts:95](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L95)

___

### networkPeers

• `Private` **networkPeers**: [*default*](network_network_peers.default.md)

Defined in: [packages/core/src/index.ts:92](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L92)

___

### paymentChannels

• `Private` **paymentChannels**: *Promise*<default\>

Defined in: [packages/core/src/index.ts:97](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L97)

___

### status

• **status**: [*NodeStatus*](../modules/index.md#nodestatus)= 'UNINITIALIZED'

Defined in: [packages/core/src/index.ts:88](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L88)

___

### strategy

• `Private` **strategy**: [*ChannelStrategy*](../interfaces/channel_strategy.channelstrategy.md)

Defined in: [packages/core/src/index.ts:91](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L91)

___

### captureRejectionSymbol

▪ `Static` `Readonly` **captureRejectionSymbol**: *typeof* [*captureRejectionSymbol*](index.default.md#capturerejectionsymbol)

Inherited from: EventEmitter.captureRejectionSymbol

Defined in: packages/core/node_modules/@types/node/events.d.ts:43

___

### captureRejections

▪ `Static` **captureRejections**: *boolean*

Sets or gets the default captureRejection value for all emitters.

Inherited from: EventEmitter.captureRejections

Defined in: packages/core/node_modules/@types/node/events.d.ts:49

___

### defaultMaxListeners

▪ `Static` **defaultMaxListeners**: *number*

Inherited from: EventEmitter.defaultMaxListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:50

___

### errorMonitor

▪ `Static` `Readonly` **errorMonitor**: *typeof* [*errorMonitor*](index.default.md#errormonitor)

This symbol shall be used to install a listener for only monitoring `'error'`
events. Listeners installed using this symbol are called before the regular
`'error'` listeners are called.

Installing a listener using this symbol does not change the behavior once an
`'error'` event is emitted, therefore the process will still crash if no
regular `'error'` listener is installed.

Inherited from: EventEmitter.errorMonitor

Defined in: packages/core/node_modules/@types/node/events.d.ts:42

## Methods

### addListener

▸ **addListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](index.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](index.default.md)

Inherited from: EventEmitter.addListener

Defined in: packages/core/node_modules/@types/node/events.d.ts:62

___

### announce

▸ `Private` **announce**(`includeRouting?`: *boolean*): *Promise*<void\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `includeRouting` | *boolean* | false |

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/index.ts:511](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L511)

___

### checkBalances

▸ `Private` **checkBalances**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/index.ts:483](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L483)

___

### closeChannel

▸ **closeChannel**(`counterparty`: *PeerId*): *Promise*<{ `receipt`: *string* ; `status`: *string*  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | *PeerId* |

**Returns:** *Promise*<{ `receipt`: *string* ; `status`: *string*  }\>

Defined in: [packages/core/src/index.ts:641](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L641)

___

### connectionReport

▸ **connectionReport**(): *Promise*<string\>

**Returns:** *Promise*<string\>

Defined in: [packages/core/src/index.ts:475](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L475)

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

Defined in: packages/core/node_modules/@types/node/events.d.ts:72

___

### eventNames

▸ **eventNames**(): (*string* \| *symbol*)[]

**Returns:** (*string* \| *symbol*)[]

Inherited from: EventEmitter.eventNames

Defined in: packages/core/node_modules/@types/node/events.d.ts:77

___

### fundChannel

▸ **fundChannel**(`counterparty`: *PeerId*, `myFund`: *BN*, `counterpartyFund`: *BN*): *Promise*<{ `channelId`: *Hash*  }\>

Fund a payment channel

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `counterparty` | *PeerId* | the counter party's peerId |
| `myFund` | *BN* | the amount to fund the channel in my favor HOPR(wei) |
| `counterpartyFund` | *BN* | the amount to fund the channel in counterparty's favor HOPR(wei) |

**Returns:** *Promise*<{ `channelId`: *Hash*  }\>

Defined in: [packages/core/src/index.ts:613](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L613)

___

### getAcknowledgedTickets

▸ **getAcknowledgedTickets**(): *Promise*<AcknowledgedTicket[]\>

**Returns:** *Promise*<AcknowledgedTicket[]\>

Defined in: [packages/core/src/index.ts:658](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L658)

___

### getAnnouncedAddresses

▸ **getAnnouncedAddresses**(`peer?`: *PeerId*): *Promise*<Multiaddr[]\>

Lists the addresses which the given node announces to other nodes

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | *PeerId* | peer to query for, default self |

**Returns:** *Promise*<Multiaddr[]\>

Defined in: [packages/core/src/index.ts:346](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L346)

___

### getBalance

▸ **getBalance**(): *Promise*<Balance\>

**Returns:** *Promise*<Balance\>

Defined in: [packages/core/src/index.ts:559](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L559)

___

### getChannelStrategy

▸ **getChannelStrategy**(): *string*

**Returns:** *string*

Defined in: [packages/core/src/index.ts:555](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L555)

___

### getChannelsOf

▸ **getChannelsOf**(`addr`: *Address*): *Promise*<ChannelEntry[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | *Address* |

**Returns:** *Promise*<ChannelEntry[]\>

Defined in: [packages/core/src/index.ts:682](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L682)

___

### getConnectedPeers

▸ **getConnectedPeers**(): *PeerId*[]

**Returns:** *PeerId*[]

Defined in: [packages/core/src/index.ts:468](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L468)

___

### getEthereumAddress

▸ **getEthereumAddress**(): *Promise*<Address\>

**Returns:** *Promise*<Address\>

Defined in: [packages/core/src/index.ts:692](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L692)

___

### getId

▸ **getId**(): *PeerId*

**Returns:** *PeerId*

Defined in: [packages/core/src/index.ts:338](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L338)

___

### getIntermediateNodes

▸ `Private` **getIntermediateNodes**(`destination`: *PeerId*): *Promise*<PeerId[]\>

Takes a destination and samples randomly intermediate nodes
that will relay that message before it reaches its destination.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `destination` | *PeerId* | instance of peerInfo that contains the peerId of the destination |

**Returns:** *Promise*<PeerId[]\>

Defined in: [packages/core/src/index.ts:708](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L708)

___

### getListeningAddresses

▸ **getListeningAddresses**(): *Multiaddr*[]

List the addresses on which the node is listening

**Returns:** *Multiaddr*[]

Defined in: [packages/core/src/index.ts:357](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L357)

___

### getMaxListeners

▸ **getMaxListeners**(): *number*

**Returns:** *number*

Inherited from: EventEmitter.getMaxListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:69

___

### getNativeBalance

▸ **getNativeBalance**(): *Promise*<NativeBalance\>

**Returns:** *Promise*<NativeBalance\>

Defined in: [packages/core/src/index.ts:564](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L564)

___

### getObservedAddresses

▸ **getObservedAddresses**(`peer`: *PeerId*): Address[]

Gets the observed addresses of a given peer.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | *PeerId* | peer to query for |

**Returns:** Address[]

Defined in: [packages/core/src/index.ts:365](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L365)

___

### getOpenChannels

▸ `Private` **getOpenChannels**(): *Promise*<RoutingChannel[]\>

**Returns:** *Promise*<RoutingChannel[]\>

Defined in: [packages/core/src/index.ts:313](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L313)

___

### getPublicKeyOf

▸ **getPublicKeyOf**(`addr`: *Address*): *Promise*<PublicKey\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | *Address* |

**Returns:** *Promise*<PublicKey\>

Defined in: [packages/core/src/index.ts:687](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L687)

___

### getVersion

▸ **getVersion**(): *any*

Returns the version of hopr-core.

**Returns:** *any*

Defined in: [packages/core/src/index.ts:320](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L320)

___

### listenerCount

▸ **listenerCount**(`event`: *string* \| *symbol*): *number*

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |

**Returns:** *number*

Inherited from: EventEmitter.listenerCount

Defined in: packages/core/node_modules/@types/node/events.d.ts:73

___

### listeners

▸ **listeners**(`event`: *string* \| *symbol*): Function[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |

**Returns:** Function[]

Inherited from: EventEmitter.listeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:70

___

### maybeLogProfilingToGCloud

▸ `Private` **maybeLogProfilingToGCloud**(): *void*

**Returns:** *void*

Defined in: [packages/core/src/index.ts:252](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L252)

___

### off

▸ **off**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](index.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](index.default.md)

Inherited from: EventEmitter.off

Defined in: packages/core/node_modules/@types/node/events.d.ts:66

___

### on

▸ **on**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](index.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](index.default.md)

Inherited from: EventEmitter.on

Defined in: packages/core/node_modules/@types/node/events.d.ts:63

___

### once

▸ **once**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](index.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](index.default.md)

Inherited from: EventEmitter.once

Defined in: packages/core/node_modules/@types/node/events.d.ts:64

___

### openChannel

▸ **openChannel**(`counterparty`: *PeerId*, `amountToFund`: *BN*): *Promise*<{ `channelId`: *Hash*  }\>

Open a payment channel

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `counterparty` | *PeerId* | the counter party's peerId |
| `amountToFund` | *BN* | the amount to fund in HOPR(wei) |

**Returns:** *Promise*<{ `channelId`: *Hash*  }\>

Defined in: [packages/core/src/index.ts:580](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L580)

___

### periodicCheck

▸ `Private` **periodicCheck**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/index.ts:497](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L497)

___

### ping

▸ **ping**(`destination`: *PeerId*): *Promise*<{ `info`: *string* ; `latency`: *number*  }\>

Ping a node.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `destination` | *PeerId* | PeerId of the node |

**Returns:** *Promise*<{ `info`: *string* ; `latency`: *number*  }\>

latency

Defined in: [packages/core/src/index.ts:454](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L454)

___

### prependListener

▸ **prependListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](index.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](index.default.md)

Inherited from: EventEmitter.prependListener

Defined in: packages/core/node_modules/@types/node/events.d.ts:75

___

### prependOnceListener

▸ **prependOnceListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](index.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](index.default.md)

Inherited from: EventEmitter.prependOnceListener

Defined in: packages/core/node_modules/@types/node/events.d.ts:76

___

### rawListeners

▸ **rawListeners**(`event`: *string* \| *symbol*): Function[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |

**Returns:** Function[]

Inherited from: EventEmitter.rawListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:71

___

### removeAllListeners

▸ **removeAllListeners**(`event?`: *string* \| *symbol*): [*default*](index.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event?` | *string* \| *symbol* |

**Returns:** [*default*](index.default.md)

Inherited from: EventEmitter.removeAllListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:67

___

### removeListener

▸ **removeListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](index.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](index.default.md)

Inherited from: EventEmitter.removeListener

Defined in: packages/core/node_modules/@types/node/events.d.ts:65

___

### sendMessage

▸ **sendMessage**(`msg`: *Uint8Array*, `destination`: *PeerId*, `getIntermediateNodesManually?`: () => *Promise*<PeerId[]\>): *Promise*<void\>

Sends a message.

**`notice`** THIS METHOD WILL SPEND YOUR ETHER.

**`notice`** This method will fail if there are not enough funds to open
the required payment channels. Please make sure that there are enough
funds controlled by the given key pair.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `msg` | *Uint8Array* | message to send |
| `destination` | *PeerId* | PeerId of the destination |
| `getIntermediateNodesManually?` | () => *Promise*<PeerId[]\> | - |

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/index.ts:382](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L382)

___

### setChannelStrategy

▸ **setChannelStrategy**(`strategy`: [*ChannelStrategyNames*](../modules/index.md#channelstrategynames)): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `strategy` | [*ChannelStrategyNames*](../modules/index.md#channelstrategynames) |

**Returns:** *void*

Defined in: [packages/core/src/index.ts:543](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L543)

___

### setMaxListeners

▸ **setMaxListeners**(`n`: *number*): [*default*](index.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `n` | *number* |

**Returns:** [*default*](index.default.md)

Inherited from: EventEmitter.setMaxListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:68

___

### smartContractInfo

▸ **smartContractInfo**(): *Promise*<string\>

**Returns:** *Promise*<string\>

Defined in: [packages/core/src/index.ts:569](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L569)

___

### start

▸ **start**(): *Promise*<void\>

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

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/index.ts:148](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L148)

___

### stop

▸ **stop**(): *Promise*<void\>

Shuts down the node and saves keys and peerBook in the database

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/index.ts:327](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L327)

___

### submitAcknowledgedTicket

▸ **submitAcknowledgedTicket**(`ackTicket`: *AcknowledgedTicket*): *Promise*<{ `ackTicket`: *AcknowledgedTicket* ; `receipt`: *string* ; `status`: ``"SUCCESS"``  } \| { `message`: *string* ; `status`: ``"FAILURE"``  } \| { `error`: *any* ; `status`: *string* = 'ERROR' }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackTicket` | *AcknowledgedTicket* |

**Returns:** *Promise*<{ `ackTicket`: *AcknowledgedTicket* ; `receipt`: *string* ; `status`: ``"SUCCESS"``  } \| { `message`: *string* ; `status`: ``"FAILURE"``  } \| { `error`: *any* ; `status`: *string* = 'ERROR' }\>

Defined in: [packages/core/src/index.ts:662](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L662)

___

### tickChannelStrategy

▸ `Private` **tickChannelStrategy**(`newChannels`: RoutingChannel[]): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `newChannels` | RoutingChannel[] |

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/index.ts:271](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L271)

___

### waitForFunds

▸ **waitForFunds**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/index.ts:721](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L721)

___

### withdraw

▸ **withdraw**(`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: *string*, `amount`: *string*): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `currency` | ``"NATIVE"`` \| ``"HOPR"`` |
| `recipient` | *string* |
| `amount` | *string* |

**Returns:** *Promise*<string\>

Defined in: [packages/core/src/index.ts:697](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L697)

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

Defined in: packages/core/node_modules/@types/node/events.d.ts:31

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

Defined in: packages/core/node_modules/@types/node/events.d.ts:28

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

Defined in: packages/core/node_modules/@types/node/events.d.ts:26

▸ `Static` **once**(`emitter`: DOMEventTarget, `event`: *string*): *Promise*<any[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `emitter` | DOMEventTarget |
| `event` | *string* |

**Returns:** *Promise*<any[]\>

Inherited from: EventEmitter.once

Defined in: packages/core/node_modules/@types/node/events.d.ts:27
