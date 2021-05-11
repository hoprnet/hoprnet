[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / default

# Class: default

## Hierarchy

- *EventEmitter*

  ↳ **default**

## Table of contents

### Constructors

- [constructor](default.md#constructor)

### Properties

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
- [getChannelsOf](default.md#getchannelsof)
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
- [removeAllListeners](default.md#removealllisteners)
- [removeListener](default.md#removelistener)
- [sendMessage](default.md#sendmessage)
- [setChannelStrategy](default.md#setchannelstrategy)
- [setMaxListeners](default.md#setmaxlisteners)
- [smartContractInfo](default.md#smartcontractinfo)
- [start](default.md#start)
- [stop](default.md#stop)
- [submitAcknowledgedTicket](default.md#submitacknowledgedticket)
- [tickChannelStrategy](default.md#tickchannelstrategy)
- [waitForFunds](default.md#waitforfunds)
- [withdraw](default.md#withdraw)
- [listenerCount](default.md#listenercount)
- [on](default.md#on)
- [once](default.md#once)

## Constructors

### constructor

\+ **new default**(`id`: *PeerId*, `options`: [*HoprOptions*](../modules.md#hoproptions)): [*default*](default.md)

Create an uninitialized Hopr Node

#### Parameters

| Name | Type |
| :------ | :------ |
| `id` | *PeerId* |
| `options` | [*HoprOptions*](../modules.md#hoproptions) |

**Returns:** [*default*](default.md)

Overrides: EventEmitter.constructor

Defined in: [packages/core/src/index.ts:97](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L97)

## Properties

### checkTimeout

• `Private` **checkTimeout**: *Timeout*

Defined in: [packages/core/src/index.ts:90](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L90)

___

### db

• `Private` **db**: *HoprDB*

Defined in: [packages/core/src/index.ts:96](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L96)

___

### forward

• `Private` **forward**: *PacketForwardInteraction*

Defined in: [packages/core/src/index.ts:94](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L94)

___

### heartbeat

• `Private` **heartbeat**: *default*

Defined in: [packages/core/src/index.ts:93](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L93)

___

### libp2p

• `Private` **libp2p**: [*LibP2P*](libp2p.md)

Defined in: [packages/core/src/index.ts:95](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L95)

___

### networkPeers

• `Private` **networkPeers**: *NetworkPeers*

Defined in: [packages/core/src/index.ts:92](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L92)

___

### paymentChannels

• `Private` **paymentChannels**: *Promise*<default\>

Defined in: [packages/core/src/index.ts:97](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L97)

___

### status

• **status**: [*NodeStatus*](../modules.md#nodestatus)= 'UNINITIALIZED'

Defined in: [packages/core/src/index.ts:88](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L88)

___

### strategy

• `Private` **strategy**: ChannelStrategy

Defined in: [packages/core/src/index.ts:91](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L91)

___

### captureRejectionSymbol

▪ `Static` `Readonly` **captureRejectionSymbol**: *typeof* [*captureRejectionSymbol*](default.md#capturerejectionsymbol)

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

▪ `Static` `Readonly` **errorMonitor**: *typeof* [*errorMonitor*](default.md#errormonitor)

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

▸ **addListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](default.md)

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

Defined in: [packages/core/src/index.ts:511](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L511)

___

### checkBalances

▸ `Private` **checkBalances**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/index.ts:483](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L483)

___

### closeChannel

▸ **closeChannel**(`counterparty`: *PeerId*): *Promise*<{ `receipt`: *string* ; `status`: *string*  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | *PeerId* |

**Returns:** *Promise*<{ `receipt`: *string* ; `status`: *string*  }\>

Defined in: [packages/core/src/index.ts:641](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L641)

___

### connectionReport

▸ **connectionReport**(): *Promise*<string\>

**Returns:** *Promise*<string\>

Defined in: [packages/core/src/index.ts:475](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L475)

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

Defined in: [packages/core/src/index.ts:613](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L613)

___

### getAcknowledgedTickets

▸ **getAcknowledgedTickets**(): *Promise*<AcknowledgedTicket[]\>

**Returns:** *Promise*<AcknowledgedTicket[]\>

Defined in: [packages/core/src/index.ts:658](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L658)

___

### getAnnouncedAddresses

▸ **getAnnouncedAddresses**(`peer?`: *PeerId*): *Promise*<Multiaddr[]\>

Lists the addresses which the given node announces to other nodes

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | *PeerId* | peer to query for, default self |

**Returns:** *Promise*<Multiaddr[]\>

Defined in: [packages/core/src/index.ts:346](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L346)

___

### getBalance

▸ **getBalance**(): *Promise*<Balance\>

**Returns:** *Promise*<Balance\>

Defined in: [packages/core/src/index.ts:559](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L559)

___

### getChannelStrategy

▸ **getChannelStrategy**(): *string*

**Returns:** *string*

Defined in: [packages/core/src/index.ts:555](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L555)

___

### getChannelsOf

▸ **getChannelsOf**(`addr`: *Address*): *Promise*<ChannelEntry[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | *Address* |

**Returns:** *Promise*<ChannelEntry[]\>

Defined in: [packages/core/src/index.ts:682](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L682)

___

### getConnectedPeers

▸ **getConnectedPeers**(): *PeerId*[]

**Returns:** *PeerId*[]

Defined in: [packages/core/src/index.ts:468](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L468)

___

### getEthereumAddress

▸ **getEthereumAddress**(): *Promise*<Address\>

**Returns:** *Promise*<Address\>

Defined in: [packages/core/src/index.ts:692](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L692)

___

### getId

▸ **getId**(): *PeerId*

**Returns:** *PeerId*

Defined in: [packages/core/src/index.ts:338](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L338)

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

Defined in: [packages/core/src/index.ts:708](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L708)

___

### getListeningAddresses

▸ **getListeningAddresses**(): *Multiaddr*[]

List the addresses on which the node is listening

**Returns:** *Multiaddr*[]

Defined in: [packages/core/src/index.ts:357](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L357)

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

Defined in: [packages/core/src/index.ts:564](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L564)

___

### getObservedAddresses

▸ **getObservedAddresses**(`peer`: *PeerId*): Address[]

Gets the observed addresses of a given peer.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peer` | *PeerId* | peer to query for |

**Returns:** Address[]

Defined in: [packages/core/src/index.ts:365](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L365)

___

### getOpenChannels

▸ `Private` **getOpenChannels**(): *Promise*<RoutingChannel[]\>

**Returns:** *Promise*<RoutingChannel[]\>

Defined in: [packages/core/src/index.ts:313](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L313)

___

### getPublicKeyOf

▸ **getPublicKeyOf**(`addr`: *Address*): *Promise*<PublicKey\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `addr` | *Address* |

**Returns:** *Promise*<PublicKey\>

Defined in: [packages/core/src/index.ts:687](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L687)

___

### getVersion

▸ **getVersion**(): *any*

Returns the version of hopr-core.

**Returns:** *any*

Defined in: [packages/core/src/index.ts:320](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L320)

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

Defined in: [packages/core/src/index.ts:252](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L252)

___

### off

▸ **off**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](default.md)

Inherited from: EventEmitter.off

Defined in: packages/core/node_modules/@types/node/events.d.ts:66

___

### on

▸ **on**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](default.md)

Inherited from: EventEmitter.on

Defined in: packages/core/node_modules/@types/node/events.d.ts:63

___

### once

▸ **once**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](default.md)

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

Defined in: [packages/core/src/index.ts:580](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L580)

___

### periodicCheck

▸ `Private` **periodicCheck**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/index.ts:497](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L497)

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

Defined in: [packages/core/src/index.ts:454](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L454)

___

### prependListener

▸ **prependListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](default.md)

Inherited from: EventEmitter.prependListener

Defined in: packages/core/node_modules/@types/node/events.d.ts:75

___

### prependOnceListener

▸ **prependOnceListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](default.md)

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

▸ **removeAllListeners**(`event?`: *string* \| *symbol*): [*default*](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event?` | *string* \| *symbol* |

**Returns:** [*default*](default.md)

Inherited from: EventEmitter.removeAllListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:67

___

### removeListener

▸ **removeListener**(`event`: *string* \| *symbol*, `listener`: (...`args`: *any*[]) => *void*): [*default*](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | *string* \| *symbol* |
| `listener` | (...`args`: *any*[]) => *void* |

**Returns:** [*default*](default.md)

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

Defined in: [packages/core/src/index.ts:382](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L382)

___

### setChannelStrategy

▸ **setChannelStrategy**(`strategy`: [*ChannelStrategyNames*](../modules.md#channelstrategynames)): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `strategy` | [*ChannelStrategyNames*](../modules.md#channelstrategynames) |

**Returns:** *void*

Defined in: [packages/core/src/index.ts:543](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L543)

___

### setMaxListeners

▸ **setMaxListeners**(`n`: *number*): [*default*](default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `n` | *number* |

**Returns:** [*default*](default.md)

Inherited from: EventEmitter.setMaxListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:68

___

### smartContractInfo

▸ **smartContractInfo**(): *Promise*<string\>

**Returns:** *Promise*<string\>

Defined in: [packages/core/src/index.ts:569](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L569)

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

Defined in: [packages/core/src/index.ts:148](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L148)

___

### stop

▸ **stop**(): *Promise*<void\>

Shuts down the node and saves keys and peerBook in the database

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/index.ts:327](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L327)

___

### submitAcknowledgedTicket

▸ **submitAcknowledgedTicket**(`ackTicket`: *AcknowledgedTicket*): *Promise*<{ `ackTicket`: *AcknowledgedTicket* ; `receipt`: *string* ; `status`: ``"SUCCESS"``  } \| { `message`: *string* ; `status`: ``"FAILURE"``  } \| { `error`: *any* ; `status`: *string* = 'ERROR' }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackTicket` | *AcknowledgedTicket* |

**Returns:** *Promise*<{ `ackTicket`: *AcknowledgedTicket* ; `receipt`: *string* ; `status`: ``"SUCCESS"``  } \| { `message`: *string* ; `status`: ``"FAILURE"``  } \| { `error`: *any* ; `status`: *string* = 'ERROR' }\>

Defined in: [packages/core/src/index.ts:662](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L662)

___

### tickChannelStrategy

▸ `Private` **tickChannelStrategy**(`newChannels`: RoutingChannel[]): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `newChannels` | RoutingChannel[] |

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/index.ts:271](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L271)

___

### waitForFunds

▸ **waitForFunds**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/index.ts:721](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L721)

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

Defined in: [packages/core/src/index.ts:697](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L697)

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
