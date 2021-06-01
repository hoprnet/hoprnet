[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / Channel

# Class: Channel

## Table of contents

### Constructors

- [constructor](channel.md#constructor)

### Properties

- [commitment](channel.md#commitment)

### Methods

- [acknowledge](channel.md#acknowledge)
- [bumpTicketIndex](channel.md#bumpticketindex)
- [createDummyTicket](channel.md#createdummyticket)
- [createTicket](channel.md#createticket)
- [finalizeClosure](channel.md#finalizeclosure)
- [fund](channel.md#fund)
- [getBalances](channel.md#getbalances)
- [getChainCommitment](channel.md#getchaincommitment)
- [getId](channel.md#getid)
- [getState](channel.md#getstate)
- [initializeClosure](channel.md#initializeclosure)
- [open](channel.md#open)
- [redeemTicket](channel.md#redeemticket)
- [generateId](channel.md#generateid)

## Constructors

### constructor

\+ **new Channel**(`self`: *PublicKey*, `counterparty`: *PublicKey*, `db`: *HoprDB*, `chain`: { `announce`: (`multiaddr`: Multiaddr) => *Promise*<string\> ; `finalizeChannelClosure`: (`counterparty`: *Address*) => *Promise*<string\> ; `fundChannel`: (`me`: *Address*, `counterparty`: *Address*, `myTotal`: *Balance*, `theirTotal`: *Balance*) => *Promise*<string\> ; `getBalance`: (`address`: *Address*) => *Promise*<Balance\> ; `getChannels`: () => *HoprChannels* ; `getGenesisBlock`: () => *number* ; `getInfo`: () => *string* ; `getLatestBlockNumber`: () => *Promise*<number\> ; `getNativeBalance`: (`address`: *Address*) => *Promise*<NativeBalance\> ; `getPrivateKey`: () => *Uint8Array* ; `getPublicKey`: () => *PublicKey* ; `getWallet`: () => *Wallet* ; `initiateChannelClosure`: (`counterparty`: *Address*) => *Promise*<string\> ; `openChannel`: (`me`: *Address*, `counterparty`: *Address*, `amount`: *Balance*) => *Promise*<string\> ; `redeemTicket`: (`counterparty`: *Address*, `ackTicket`: *AcknowledgedTicket*, `ticket`: *Ticket*) => *Promise*<string\> ; `setCommitment`: (`counterparty`: *Address*, `comm`: *Hash*) => *Promise*<string\> ; `subscribeBlock`: (`cb`: *any*) => *JsonRpcProvider* \| *WebSocketProvider* ; `subscribeChannelEvents`: (`cb`: *any*) => *HoprChannels* ; `subscribeError`: (`cb`: *any*) => *void* ; `unsubscribe`: () => *void* ; `waitUntilReady`: () => *Promise*<Network\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: *string*, `amount`: *string*) => *Promise*<string\>  }, `indexer`: [*Indexer*](indexer.md), `privateKey`: *Uint8Array*): [*Channel*](channel.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `self` | *PublicKey* |
| `counterparty` | *PublicKey* |
| `db` | *HoprDB* |
| `chain` | *object* |
| `chain.announce` | (`multiaddr`: Multiaddr) => *Promise*<string\> |
| `chain.finalizeChannelClosure` | (`counterparty`: *Address*) => *Promise*<string\> |
| `chain.fundChannel` | (`me`: *Address*, `counterparty`: *Address*, `myTotal`: *Balance*, `theirTotal`: *Balance*) => *Promise*<string\> |
| `chain.getBalance` | (`address`: *Address*) => *Promise*<Balance\> |
| `chain.getChannels` | () => *HoprChannels* |
| `chain.getGenesisBlock` | () => *number* |
| `chain.getInfo` | () => *string* |
| `chain.getLatestBlockNumber` | () => *Promise*<number\> |
| `chain.getNativeBalance` | (`address`: *Address*) => *Promise*<NativeBalance\> |
| `chain.getPrivateKey` | () => *Uint8Array* |
| `chain.getPublicKey` | () => *PublicKey* |
| `chain.getWallet` | () => *Wallet* |
| `chain.initiateChannelClosure` | (`counterparty`: *Address*) => *Promise*<string\> |
| `chain.openChannel` | (`me`: *Address*, `counterparty`: *Address*, `amount`: *Balance*) => *Promise*<string\> |
| `chain.redeemTicket` | (`counterparty`: *Address*, `ackTicket`: *AcknowledgedTicket*, `ticket`: *Ticket*) => *Promise*<string\> |
| `chain.setCommitment` | (`counterparty`: *Address*, `comm`: *Hash*) => *Promise*<string\> |
| `chain.subscribeBlock` | (`cb`: *any*) => *JsonRpcProvider* \| *WebSocketProvider* |
| `chain.subscribeChannelEvents` | (`cb`: *any*) => *HoprChannels* |
| `chain.subscribeError` | (`cb`: *any*) => *void* |
| `chain.unsubscribe` | () => *void* |
| `chain.waitUntilReady` | () => *Promise*<Network\> |
| `chain.withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: *string*, `amount`: *string*) => *Promise*<string\> |
| `indexer` | [*Indexer*](indexer.md) |
| `privateKey` | *Uint8Array* |

**Returns:** [*Channel*](channel.md)

Defined in: [core-ethereum/src/channel.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L25)

## Properties

### commitment

• `Private` **commitment**: *Commitment*

Defined in: [core-ethereum/src/channel.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L25)

## Methods

### acknowledge

▸ **acknowledge**(`unacknowledgedTicket`: *UnacknowledgedTicket*, `acknowledgement`: *HalfKey*): *Promise*<AcknowledgedTicket\>

Reserve a preImage for the given ticket if it is a winning ticket.

#### Parameters

| Name | Type |
| :------ | :------ |
| `unacknowledgedTicket` | *UnacknowledgedTicket* |
| `acknowledgement` | *HalfKey* |

**Returns:** *Promise*<AcknowledgedTicket\>

Defined in: [core-ethereum/src/channel.ts:53](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L53)

___

### bumpTicketIndex

▸ `Private` **bumpTicketIndex**(`channelState`: *ChannelEntry*): *Promise*<UINT256\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelState` | *ChannelEntry* |

**Returns:** *Promise*<UINT256\>

Defined in: [core-ethereum/src/channel.ts:155](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L155)

___

### createDummyTicket

▸ **createDummyTicket**(`challenge`: *Challenge*): *Ticket*

Creates a ticket that is sent next to the packet to the last node.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `challenge` | *Challenge* | dummy challenge, potential no valid response known |

**Returns:** *Ticket*

a ticket without any value

Defined in: [core-ethereum/src/channel.ts:207](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L207)

___

### createTicket

▸ **createTicket**(`amount`: *Balance*, `challenge`: *Challenge*, `winProb`: *BN*): *Promise*<Ticket\>

Creates a signed ticket that includes the given amount of
tokens

**`dev`** Due to a missing feature, namely ECMUL, in Ethereum, the
challenge is given as an Ethereum address because the signature
recovery algorithm is used to perform an EC-point multiplication.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `amount` | *Balance* | value of the ticket |
| `challenge` | *Challenge* | challenge to solve in order to redeem the ticket |
| `winProb` | *BN* | the winning probability to use |

**Returns:** *Promise*<Ticket\>

a signed ticket

Defined in: [core-ethereum/src/channel.ts:183](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L183)

___

### finalizeClosure

▸ **finalizeClosure**(): *Promise*<string\>

**Returns:** *Promise*<string\>

Defined in: [core-ethereum/src/channel.ts:145](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L145)

___

### fund

▸ **fund**(`myFund`: *Balance*, `counterpartyFund`: *Balance*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `myFund` | *Balance* |
| `counterpartyFund` | *Balance* |

**Returns:** *Promise*<void\>

Defined in: [core-ethereum/src/channel.ts:106](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L106)

___

### getBalances

▸ **getBalances**(): *Promise*<{ `counterparty`: *Balance* ; `self`: *Balance*  }\>

**Returns:** *Promise*<{ `counterparty`: *Balance* ; `self`: *Balance*  }\>

Defined in: [core-ethereum/src/channel.ts:94](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L94)

___

### getChainCommitment

▸ **getChainCommitment**(): *Promise*<Hash\>

**Returns:** *Promise*<Hash\>

Defined in: [core-ethereum/src/channel.ts:81](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L81)

___

### getId

▸ **getId**(): *Hash*

**Returns:** *Hash*

Defined in: [core-ethereum/src/channel.ts:77](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L77)

___

### getState

▸ **getState**(): *Promise*<ChannelEntry\>

**Returns:** *Promise*<ChannelEntry\>

Defined in: [core-ethereum/src/channel.ts:85](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L85)

___

### initializeClosure

▸ **initializeClosure**(): *Promise*<string\>

**Returns:** *Promise*<string\>

Defined in: [core-ethereum/src/channel.ts:136](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L136)

___

### open

▸ **open**(`fundAmount`: *Balance*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `fundAmount` | *Balance* |

**Returns:** *Promise*<void\>

Defined in: [core-ethereum/src/channel.ts:117](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L117)

___

### redeemTicket

▸ **redeemTicket**(`ackTicket`: *AcknowledgedTicket*): *Promise*<[*RedeemTicketResponse*](../modules.md#redeemticketresponse)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackTicket` | *AcknowledgedTicket* |

**Returns:** *Promise*<[*RedeemTicketResponse*](../modules.md#redeemticketresponse)\>

Defined in: [core-ethereum/src/channel.ts:221](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L221)

___

### generateId

▸ `Static` **generateId**(`self`: *Address*, `counterparty`: *Address*): *Hash*

#### Parameters

| Name | Type |
| :------ | :------ |
| `self` | *Address* |
| `counterparty` | *Address* |

**Returns:** *Hash*

Defined in: [core-ethereum/src/channel.ts:44](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L44)
