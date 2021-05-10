[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / default

# Class: default

## Hierarchy

- _EventEmitter_

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

\+ **new default**(`id`: _PeerId_, `options`: [_HoprOptions_](../modules.md#hoproptions)): [_default_](default.md)

Create an uninitialized Hopr Node

#### Parameters

| Name      | Type                                       |
| :-------- | :----------------------------------------- |
| `id`      | _PeerId_                                   |
| `options` | [_HoprOptions_](../modules.md#hoproptions) |

**Returns:** [_default_](default.md)

Overrides: EventEmitter.constructor

Defined in: [packages/core/src/index.ts:97](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L97)

## Properties

### checkTimeout

• `Private` **checkTimeout**: _Timeout_

Defined in: [packages/core/src/index.ts:90](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L90)

---

### db

• `Private` **db**: _HoprDB_

Defined in: [packages/core/src/index.ts:96](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L96)

---

### forward

• `Private` **forward**: _PacketForwardInteraction_

Defined in: [packages/core/src/index.ts:94](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L94)

---

### heartbeat

• `Private` **heartbeat**: _default_

Defined in: [packages/core/src/index.ts:93](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L93)

---

### libp2p

• `Private` **libp2p**: [_LibP2P_](libp2p.md)

Defined in: [packages/core/src/index.ts:95](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L95)

---

### networkPeers

• `Private` **networkPeers**: _NetworkPeers_

Defined in: [packages/core/src/index.ts:92](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L92)

---

### paymentChannels

• `Private` **paymentChannels**: _Promise_<default\>

Defined in: [packages/core/src/index.ts:97](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L97)

---

### status

• **status**: [_NodeStatus_](../modules.md#nodestatus)= 'UNINITIALIZED'

Defined in: [packages/core/src/index.ts:88](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L88)

---

### strategy

• `Private` **strategy**: ChannelStrategy

Defined in: [packages/core/src/index.ts:91](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L91)

---

### captureRejectionSymbol

▪ `Static` `Readonly` **captureRejectionSymbol**: _typeof_ [_captureRejectionSymbol_](default.md#capturerejectionsymbol)

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

▪ `Static` `Readonly` **errorMonitor**: _typeof_ [_errorMonitor_](default.md#errormonitor)

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

▸ **addListener**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_default_](default.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_default_](default.md)

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

Defined in: [packages/core/src/index.ts:511](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L511)

---

### checkBalances

▸ `Private` **checkBalances**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/index.ts:483](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L483)

---

### closeChannel

▸ **closeChannel**(`counterparty`: _PeerId_): _Promise_<{ `receipt`: _string_ ; `status`: _string_ }\>

#### Parameters

| Name           | Type     |
| :------------- | :------- |
| `counterparty` | _PeerId_ |

**Returns:** _Promise_<{ `receipt`: _string_ ; `status`: _string_ }\>

Defined in: [packages/core/src/index.ts:641](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L641)

---

### connectionReport

▸ **connectionReport**(): _Promise_<string\>

**Returns:** _Promise_<string\>

Defined in: [packages/core/src/index.ts:475](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L475)

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

Defined in: [packages/core/src/index.ts:613](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L613)

---

### getAcknowledgedTickets

▸ **getAcknowledgedTickets**(): _Promise_<AcknowledgedTicket[]\>

**Returns:** _Promise_<AcknowledgedTicket[]\>

Defined in: [packages/core/src/index.ts:658](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L658)

---

### getAnnouncedAddresses

▸ **getAnnouncedAddresses**(`peer?`: _PeerId_): _Promise_<Multiaddr[]\>

Lists the addresses which the given node announces to other nodes

#### Parameters

| Name   | Type     | Description                     |
| :----- | :------- | :------------------------------ |
| `peer` | _PeerId_ | peer to query for, default self |

**Returns:** _Promise_<Multiaddr[]\>

Defined in: [packages/core/src/index.ts:346](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L346)

---

### getBalance

▸ **getBalance**(): _Promise_<Balance\>

**Returns:** _Promise_<Balance\>

Defined in: [packages/core/src/index.ts:559](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L559)

---

### getChannelStrategy

▸ **getChannelStrategy**(): _string_

**Returns:** _string_

Defined in: [packages/core/src/index.ts:555](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L555)

---

### getChannelsOf

▸ **getChannelsOf**(`addr`: _Address_): _Promise_<ChannelEntry[]\>

#### Parameters

| Name   | Type      |
| :----- | :-------- |
| `addr` | _Address_ |

**Returns:** _Promise_<ChannelEntry[]\>

Defined in: [packages/core/src/index.ts:682](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L682)

---

### getConnectedPeers

▸ **getConnectedPeers**(): _PeerId_[]

**Returns:** _PeerId_[]

Defined in: [packages/core/src/index.ts:468](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L468)

---

### getEthereumAddress

▸ **getEthereumAddress**(): _Promise_<Address\>

**Returns:** _Promise_<Address\>

Defined in: [packages/core/src/index.ts:692](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L692)

---

### getId

▸ **getId**(): _PeerId_

**Returns:** _PeerId_

Defined in: [packages/core/src/index.ts:338](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L338)

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

Defined in: [packages/core/src/index.ts:708](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L708)

---

### getListeningAddresses

▸ **getListeningAddresses**(): _Multiaddr_[]

List the addresses on which the node is listening

**Returns:** _Multiaddr_[]

Defined in: [packages/core/src/index.ts:357](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L357)

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

Defined in: [packages/core/src/index.ts:564](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L564)

---

### getObservedAddresses

▸ **getObservedAddresses**(`peer`: _PeerId_): Address[]

Gets the observed addresses of a given peer.

#### Parameters

| Name   | Type     | Description       |
| :----- | :------- | :---------------- |
| `peer` | _PeerId_ | peer to query for |

**Returns:** Address[]

Defined in: [packages/core/src/index.ts:365](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L365)

---

### getOpenChannels

▸ `Private` **getOpenChannels**(): _Promise_<RoutingChannel[]\>

**Returns:** _Promise_<RoutingChannel[]\>

Defined in: [packages/core/src/index.ts:313](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L313)

---

### getPublicKeyOf

▸ **getPublicKeyOf**(`addr`: _Address_): _Promise_<PublicKey\>

#### Parameters

| Name   | Type      |
| :----- | :-------- |
| `addr` | _Address_ |

**Returns:** _Promise_<PublicKey\>

Defined in: [packages/core/src/index.ts:687](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L687)

---

### getVersion

▸ **getVersion**(): _any_

Returns the version of hopr-core.

**Returns:** _any_

Defined in: [packages/core/src/index.ts:320](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L320)

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

Defined in: [packages/core/src/index.ts:252](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L252)

---

### off

▸ **off**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_default_](default.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_default_](default.md)

Inherited from: EventEmitter.off

Defined in: packages/core/node_modules/@types/node/events.d.ts:66

---

### on

▸ **on**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_default_](default.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_default_](default.md)

Inherited from: EventEmitter.on

Defined in: packages/core/node_modules/@types/node/events.d.ts:63

---

### once

▸ **once**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_default_](default.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_default_](default.md)

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

Defined in: [packages/core/src/index.ts:580](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L580)

---

### periodicCheck

▸ `Private` **periodicCheck**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/index.ts:497](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L497)

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

Defined in: [packages/core/src/index.ts:454](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L454)

---

### prependListener

▸ **prependListener**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_default_](default.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_default_](default.md)

Inherited from: EventEmitter.prependListener

Defined in: packages/core/node_modules/@types/node/events.d.ts:75

---

### prependOnceListener

▸ **prependOnceListener**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_default_](default.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_default_](default.md)

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

▸ **removeAllListeners**(`event?`: _string_ \| _symbol_): [_default_](default.md)

#### Parameters

| Name     | Type                 |
| :------- | :------------------- |
| `event?` | _string_ \| _symbol_ |

**Returns:** [_default_](default.md)

Inherited from: EventEmitter.removeAllListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:67

---

### removeListener

▸ **removeListener**(`event`: _string_ \| _symbol_, `listener`: (...`args`: _any_[]) => _void_): [_default_](default.md)

#### Parameters

| Name       | Type                           |
| :--------- | :----------------------------- |
| `event`    | _string_ \| _symbol_           |
| `listener` | (...`args`: _any_[]) => _void_ |

**Returns:** [_default_](default.md)

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

Defined in: [packages/core/src/index.ts:382](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L382)

---

### setChannelStrategy

▸ **setChannelStrategy**(`strategy`: [_ChannelStrategyNames_](../modules.md#channelstrategynames)): _void_

#### Parameters

| Name       | Type                                                         |
| :--------- | :----------------------------------------------------------- |
| `strategy` | [_ChannelStrategyNames_](../modules.md#channelstrategynames) |

**Returns:** _void_

Defined in: [packages/core/src/index.ts:543](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L543)

---

### setMaxListeners

▸ **setMaxListeners**(`n`: _number_): [_default_](default.md)

#### Parameters

| Name | Type     |
| :--- | :------- |
| `n`  | _number_ |

**Returns:** [_default_](default.md)

Inherited from: EventEmitter.setMaxListeners

Defined in: packages/core/node_modules/@types/node/events.d.ts:68

---

### smartContractInfo

▸ **smartContractInfo**(): _Promise_<string\>

**Returns:** _Promise_<string\>

Defined in: [packages/core/src/index.ts:569](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L569)

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

Defined in: [packages/core/src/index.ts:148](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L148)

---

### stop

▸ **stop**(): _Promise_<void\>

Shuts down the node and saves keys and peerBook in the database

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/index.ts:327](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L327)

---

### submitAcknowledgedTicket

▸ **submitAcknowledgedTicket**(`ackTicket`: _AcknowledgedTicket_): _Promise_<{ `ackTicket`: _AcknowledgedTicket_ ; `receipt`: _string_ ; `status`: `"SUCCESS"` } \| { `message`: _string_ ; `status`: `"FAILURE"` } \| { `error`: _any_ ; `status`: _string_ = 'ERROR' }\>

#### Parameters

| Name        | Type                 |
| :---------- | :------------------- |
| `ackTicket` | _AcknowledgedTicket_ |

**Returns:** _Promise_<{ `ackTicket`: _AcknowledgedTicket_ ; `receipt`: _string_ ; `status`: `"SUCCESS"` } \| { `message`: _string_ ; `status`: `"FAILURE"` } \| { `error`: _any_ ; `status`: _string_ = 'ERROR' }\>

Defined in: [packages/core/src/index.ts:662](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L662)

---

### tickChannelStrategy

▸ `Private` **tickChannelStrategy**(`newChannels`: RoutingChannel[]): _Promise_<void\>

#### Parameters

| Name          | Type             |
| :------------ | :--------------- |
| `newChannels` | RoutingChannel[] |

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/index.ts:271](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L271)

---

### waitForFunds

▸ **waitForFunds**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/index.ts:721](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L721)

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

Defined in: [packages/core/src/index.ts:697](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L697)

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
