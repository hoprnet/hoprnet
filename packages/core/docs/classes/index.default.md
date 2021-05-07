[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / [index](../modules/index.md) / default

# Class: default

[index](../modules/index.md).default

## Hierarchy

- _EventEmitter_

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

\+ **new default**(`id`: _PeerId_, `options`: [_HoprOptions_](../modules/index.md#hoproptions)): [_default_](index.default.md)

Create an uninitialized Hopr Node

#### Parameters

| Name      | Type                                             |
| :-------- | :----------------------------------------------- |
| `id`      | _PeerId_                                         |
| `options` | [_HoprOptions_](../modules/index.md#hoproptions) |

**Returns:** [_default_](index.default.md)

Overrides: EventEmitter.constructor

Defined in: [packages/core/src/index.ts:97](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L97)

## Properties

### checkTimeout

• `Private` **checkTimeout**: _Timeout_

Defined in: [packages/core/src/index.ts:90](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L90)

---

### db

• `Private` **db**: _HoprDB_

Defined in: [packages/core/src/index.ts:96](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L96)

---

### forward

• `Private` **forward**: [_PacketForwardInteraction_](interactions_packet_forward.packetforwardinteraction.md)

Defined in: [packages/core/src/index.ts:94](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L94)

---

### heartbeat

• `Private` **heartbeat**: [_default_](network_heartbeat.default.md)

Defined in: [packages/core/src/index.ts:93](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L93)

---

### libp2p

• `Private` **libp2p**: [_LibP2P_](index.libp2p-1.md)

Defined in: [packages/core/src/index.ts:95](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L95)

---

### networkPeers

• `Private` **networkPeers**: [_default_](network_network_peers.default.md)

Defined in: [packages/core/src/index.ts:92](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L92)

---

### paymentChannels

• `Private` **paymentChannels**: _Promise_<default\>

Defined in: [packages/core/src/index.ts:97](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L97)

---

### status

• **status**: [_NodeStatus_](../modules/index.md#nodestatus)= 'UNINITIALIZED'

Defined in: [packages/core/src/index.ts:88](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L88)

---

### strategy

• `Private` **strategy**: [_ChannelStrategy_](../interfaces/channel_strategy.channelstrategy.md)

Defined in: [packages/core/src/index.ts:91](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L91)

---

### captureRejectionSymbol

▪ `Static` `Readonly` **captureRejectionSymbol**: _typeof_ [_captureRejectionSymbol_](index.default.md#capturerejectionsymbol)

Inherited from: EventEmitter.captureRejectionSymbol

Defined in: packages/core/node_modules/@types/node/events.d.ts:43

---

### captureRejections

▪ `Static` **captureRejections**: _boolean_

Sets or gets the default captureRejection value for all emitters.

Inherited from: EventEmitter.captureRejections

Defined in: packages/core/node_modules/@types/node/events.d.ts:49

---

### defaultMaxListeners

▪ `Static` **defaultMaxListeners**: _number_

Inherited from: EventEmitter.defaultMaxListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:50

---

### errorMonitor

▪ `Static` `Readonly` **errorMonitor**: _typeof_ [_errorMonitor_](index.default.md#errormonitor)

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

▸ **addListener**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_default_](index.default.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_default_](index.default.md)

Inherited from: EventEmitter.addListener

Defined in: packages/core/node_modules/@types/node/events.d.ts:62

---

### announce

▸ `Private` **announce**(`includeRouting?`: _boolean_): _Promise_<void\>

#### Parameters

| Name             | Type      | Default value |
| :--------------- | :-------- | :------------ |
| `includeRouting` | _boolean_ | false         |

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/index.ts:511](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L511)

---

### checkBalances

▸ `Private` **checkBalances**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/index.ts:483](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L483)

---

### closeChannel

▸ **closeChannel**(`counterparty`: _PeerId_): _Promise_<{ `receipt`: _string_ ; `status`: _string_ }\>

#### Parameters

| Name           | Type     |
| :------------- | :------- |
| `counterparty` | _PeerId_ |

**Returns:** _Promise_<{ `receipt`: _string_ ; `status`: _string_ }\>

Defined in: [packages/core/src/index.ts:641](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L641)

---

### connectionReport

▸ **connectionReport**(): _Promise_<string\>

**Returns:** _Promise_<string\>

Defined in: [packages/core/src/index.ts:475](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L475)

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

Defined in: packages/core/node_modules/@types/node/events.d.ts:72

---

### eventNames

▸ **eventNames**(): (_string_ \| _symbol_)[]

**Returns:** (_string_ \| _symbol_)[]

Inherited from: EventEmitter.eventNames

Defined in: packages/core/node_modules/@types/node/events.d.ts:77

---

### fundChannel

▸ **fundChannel**(`counterparty`: _PeerId_, `myFund`: _BN_, `counterpartyFund`: _BN_): _Promise_<{ `channelId`: _Hash_ }\>

Fund a payment channel

#### Parameters

| Name               | Type     | Description                                                      |
| :----------------- | :------- | :--------------------------------------------------------------- |
| `counterparty`     | _PeerId_ | the counter party's peerId                                       |
| `myFund`           | _BN_     | the amount to fund the channel in my favor HOPR(wei)             |
| `counterpartyFund` | _BN_     | the amount to fund the channel in counterparty's favor HOPR(wei) |

**Returns:** _Promise_<{ `channelId`: _Hash_ }\>

Defined in: [packages/core/src/index.ts:613](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L613)

---

### getAcknowledgedTickets

▸ **getAcknowledgedTickets**(): _Promise_<AcknowledgedTicket[]\>

**Returns:** _Promise_<AcknowledgedTicket[]\>

Defined in: [packages/core/src/index.ts:658](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L658)

---

### getAnnouncedAddresses

▸ **getAnnouncedAddresses**(`peer?`: _PeerId_): _Promise_<Multiaddr[]\>

Lists the addresses which the given node announces to other nodes

#### Parameters

| Name   | Type     | Description                     |
| :----- | :------- | :------------------------------ |
| `peer` | _PeerId_ | peer to query for, default self |

**Returns:** _Promise_<Multiaddr[]\>

Defined in: [packages/core/src/index.ts:346](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L346)

---

### getBalance

▸ **getBalance**(): _Promise_<Balance\>

**Returns:** _Promise_<Balance\>

Defined in: [packages/core/src/index.ts:559](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L559)

---

### getChannelStrategy

▸ **getChannelStrategy**(): _string_

**Returns:** _string_

Defined in: [packages/core/src/index.ts:555](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L555)

---

### getChannelsOf

▸ **getChannelsOf**(`addr`: _Address_): _Promise_<ChannelEntry[]\>

#### Parameters

| Name   | Type      |
| :----- | :-------- |
| `addr` | _Address_ |

**Returns:** _Promise_<ChannelEntry[]\>

Defined in: [packages/core/src/index.ts:682](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L682)

---

### getConnectedPeers

▸ **getConnectedPeers**(): _PeerId_[]

**Returns:** _PeerId_[]

Defined in: [packages/core/src/index.ts:468](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L468)

---

### getEthereumAddress

▸ **getEthereumAddress**(): _Promise_<Address\>

**Returns:** _Promise_<Address\>

Defined in: [packages/core/src/index.ts:692](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L692)

---

### getId

▸ **getId**(): _PeerId_

**Returns:** _PeerId_

Defined in: [packages/core/src/index.ts:338](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L338)

---

### getIntermediateNodes

▸ `Private` **getIntermediateNodes**(`destination`: _PeerId_): _Promise_<PeerId[]\>

Takes a destination and samples randomly intermediate nodes
that will relay that message before it reaches its destination.

#### Parameters

| Name          | Type     | Description                                                      |
| :------------ | :------- | :--------------------------------------------------------------- |
| `destination` | _PeerId_ | instance of peerInfo that contains the peerId of the destination |

**Returns:** _Promise_<PeerId[]\>

Defined in: [packages/core/src/index.ts:708](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L708)

---

### getListeningAddresses

▸ **getListeningAddresses**(): _Multiaddr_[]

List the addresses on which the node is listening

**Returns:** _Multiaddr_[]

Defined in: [packages/core/src/index.ts:357](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L357)

---

### getMaxListeners

▸ **getMaxListeners**(): _number_

**Returns:** _number_

Inherited from: EventEmitter.getMaxListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:69

---

### getNativeBalance

▸ **getNativeBalance**(): _Promise_<NativeBalance\>

**Returns:** _Promise_<NativeBalance\>

Defined in: [packages/core/src/index.ts:564](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L564)

---

### getObservedAddresses

▸ **getObservedAddresses**(`peer`: _PeerId_): Address[]

Gets the observed addresses of a given peer.

#### Parameters

| Name   | Type     | Description       |
| :----- | :------- | :---------------- |
| `peer` | _PeerId_ | peer to query for |

**Returns:** Address[]

Defined in: [packages/core/src/index.ts:365](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L365)

---

### getOpenChannels

▸ `Private` **getOpenChannels**(): _Promise_<RoutingChannel[]\>

**Returns:** _Promise_<RoutingChannel[]\>

Defined in: [packages/core/src/index.ts:313](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L313)

---

### getPublicKeyOf

▸ **getPublicKeyOf**(`addr`: _Address_): _Promise_<PublicKey\>

#### Parameters

| Name   | Type      |
| :----- | :-------- |
| `addr` | _Address_ |

**Returns:** _Promise_<PublicKey\>

Defined in: [packages/core/src/index.ts:687](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L687)

---

### getVersion

▸ **getVersion**(): _any_

Returns the version of hopr-core.

**Returns:** _any_

Defined in: [packages/core/src/index.ts:320](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L320)

---

### listenerCount

▸ **listenerCount**(`event`: _string_ \| _symbol_): _number_

#### Parameters

| Name    | Type                 |
| :------ | :------------------- |
| `event` | _string_ \| _symbol_ |

**Returns:** _number_

Inherited from: EventEmitter.listenerCount

Defined in: packages/core/node_modules/@types/node/events.d.ts:73

---

### listeners

▸ **listeners**(`event`: _string_ \| _symbol_): Function[]

#### Parameters

| Name    | Type                 |
| :------ | :------------------- |
| `event` | _string_ \| _symbol_ |

**Returns:** Function[]

Inherited from: EventEmitter.listeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:70

---

### maybeLogProfilingToGCloud

▸ `Private` **maybeLogProfilingToGCloud**(): _void_

**Returns:** _void_

Defined in: [packages/core/src/index.ts:252](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L252)

---

### off

▸ **off**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_default_](index.default.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_default_](index.default.md)

Inherited from: EventEmitter.off

Defined in: packages/core/node_modules/@types/node/events.d.ts:66

---

### on

▸ **on**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_default_](index.default.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_default_](index.default.md)

Inherited from: EventEmitter.on

Defined in: packages/core/node_modules/@types/node/events.d.ts:63

---

### once

▸ **once**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_default_](index.default.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_default_](index.default.md)

Inherited from: EventEmitter.once

Defined in: packages/core/node_modules/@types/node/events.d.ts:64

---

### openChannel

▸ **openChannel**(`counterparty`: _PeerId_, `amountToFund`: _BN_): _Promise_<{ `channelId`: _Hash_ }\>

Open a payment channel

#### Parameters

| Name           | Type     | Description                     |
| :------------- | :------- | :------------------------------ |
| `counterparty` | _PeerId_ | the counter party's peerId      |
| `amountToFund` | _BN_     | the amount to fund in HOPR(wei) |

**Returns:** _Promise_<{ `channelId`: _Hash_ }\>

Defined in: [packages/core/src/index.ts:580](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L580)

---

### periodicCheck

▸ `Private` **periodicCheck**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/index.ts:497](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L497)

---

### ping

▸ **ping**(`destination`: _PeerId_): _Promise_<{ `info`: _string_ ; `latency`: _number_ }\>

Ping a node.

#### Parameters

| Name          | Type     | Description        |
| :------------ | :------- | :----------------- |
| `destination` | _PeerId_ | PeerId of the node |

**Returns:** _Promise_<{ `info`: _string_ ; `latency`: _number_ }\>

latency

Defined in: [packages/core/src/index.ts:454](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L454)

---

### prependListener

▸ **prependListener**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_default_](index.default.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_default_](index.default.md)

Inherited from: EventEmitter.prependListener

Defined in: packages/core/node_modules/@types/node/events.d.ts:75

---

### prependOnceListener

▸ **prependOnceListener**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_default_](index.default.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_default_](index.default.md)

Inherited from: EventEmitter.prependOnceListener

Defined in: packages/core/node_modules/@types/node/events.d.ts:76

---

### rawListeners

▸ **rawListeners**(`event`: _string_ \| _symbol_): Function[]

#### Parameters

| Name    | Type                 |
| :------ | :------------------- |
| `event` | _string_ \| _symbol_ |

**Returns:** Function[]

Inherited from: EventEmitter.rawListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:71

---

### removeAllListeners

▸ **removeAllListeners**(`event?`: _string_ \| _symbol_): [_default_](index.default.md)

#### Parameters

| Name     | Type                 |
| :------- | :------------------- |
| `event?` | _string_ \| _symbol_ |

**Returns:** [_default_](index.default.md)

Inherited from: EventEmitter.removeAllListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:67

---

### removeListener

▸ **removeListener**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_default_](index.default.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_default_](index.default.md)

Inherited from: EventEmitter.removeListener

Defined in: packages/core/node_modules/@types/node/events.d.ts:65

---

### sendMessage

▸ **sendMessage**(`msg`: _Uint8Array_, `destination`: _PeerId_, `getIntermediateNodesManually?`: () => _Promise_<PeerId[]\>): _Promise_<void\>

Sends a message.

**`notice`** THIS METHOD WILL SPEND YOUR ETHER.

**`notice`** This method will fail if there are not enough funds to open
the required payment channels. Please make sure that there are enough
funds controlled by the given key pair.

#### Parameters

| Name                            | Type                       | Description               |
| :------------------------------ | :------------------------- | :------------------------ |
| `msg`                           | _Uint8Array_               | message to send           |
| `destination`                   | _PeerId_                   | PeerId of the destination |
| `getIntermediateNodesManually?` | () => _Promise_<PeerId[]\> | -                         |

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/index.ts:382](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L382)

---

### setChannelStrategy

▸ **setChannelStrategy**(`strategy`: [_ChannelStrategyNames_](../modules/index.md#channelstrategynames)): _void_

#### Parameters

| Name       | Type                                                               |
| :--------- | :----------------------------------------------------------------- |
| `strategy` | [_ChannelStrategyNames_](../modules/index.md#channelstrategynames) |

**Returns:** _void_

Defined in: [packages/core/src/index.ts:543](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L543)

---

### setMaxListeners

▸ **setMaxListeners**(`n`: _number_): [_default_](index.default.md)

#### Parameters

| Name | Type     |
| :--- | :------- |
| `n`  | _number_ |

**Returns:** [_default_](index.default.md)

Inherited from: EventEmitter.setMaxListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:68

---

### smartContractInfo

▸ **smartContractInfo**(): _Promise_<string\>

**Returns:** _Promise_<string\>

Defined in: [packages/core/src/index.ts:569](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L569)

---

### start

▸ **start**(): _Promise_<void\>

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

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/index.ts:148](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L148)

---

### stop

▸ **stop**(): _Promise_<void\>

Shuts down the node and saves keys and peerBook in the database

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/index.ts:327](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L327)

---

### submitAcknowledgedTicket

▸ **submitAcknowledgedTicket**(`ackTicket`: _AcknowledgedTicket_): _Promise_<{ `ackTicket`: _AcknowledgedTicket_ ; `receipt`: _string_ ; `status`: `"SUCCESS"` } \| { `message`: _string_ ; `status`: `"FAILURE"` } \| { `error`: _any_ ; `status`: _string_ = 'ERROR' }\>

#### Parameters

| Name        | Type                 |
| :---------- | :------------------- |
| `ackTicket` | _AcknowledgedTicket_ |

**Returns:** _Promise_<{ `ackTicket`: _AcknowledgedTicket_ ; `receipt`: _string_ ; `status`: `"SUCCESS"` } \| { `message`: _string_ ; `status`: `"FAILURE"` } \| { `error`: _any_ ; `status`: _string_ = 'ERROR' }\>

Defined in: [packages/core/src/index.ts:662](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L662)

---

### tickChannelStrategy

▸ `Private` **tickChannelStrategy**(`newChannels`: RoutingChannel[]): _Promise_<void\>

#### Parameters

| Name          | Type             |
| :------------ | :--------------- |
| `newChannels` | RoutingChannel[] |

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/index.ts:271](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L271)

---

### waitForFunds

▸ **waitForFunds**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/index.ts:721](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L721)

---

### withdraw

▸ **withdraw**(`currency`: `"NATIVE"` \| `"HOPR"`, `recipient`: _string_, `amount`: _string_): _Promise_<string\>

#### Parameters

| Name        | Type                   |
| :---------- | :--------------------- |
| `currency`  | `"NATIVE"` \| `"HOPR"` |
| `recipient` | _string_               |
| `amount`    | _string_               |

**Returns:** _Promise_<string\>

Defined in: [packages/core/src/index.ts:697](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L697)

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

Defined in: packages/core/node_modules/@types/node/events.d.ts:31

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

Defined in: packages/core/node_modules/@types/node/events.d.ts:28

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

Defined in: packages/core/node_modules/@types/node/events.d.ts:26

▸ `Static` **once**(`emitter`: DOMEventTarget, `event`: _string_): _Promise_<any[]\>

#### Parameters

| Name      | Type           |
| :-------- | :------------- |
| `emitter` | DOMEventTarget |
| `event`   | _string_       |

**Returns:** _Promise_<any[]\>

Inherited from: EventEmitter.once

Defined in: packages/core/node_modules/@types/node/events.d.ts:27
