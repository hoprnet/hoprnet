[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / Channel

# Class: Channel

## Table of contents

### Constructors

- [constructor](channel.md#constructor)

### Properties

- [commitment](channel.md#commitment)
- [index](channel.md#index)

### Methods

- [acknowledge](channel.md#acknowledge)
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

\+ **new Channel**(`self`: *PublicKey*, `counterparty`: *PublicKey*, `db`: *HoprDB*, `chain`: { `announce`: (`multiaddr`: Multiaddr) => *Promise*<string\> ; `finalizeChannelClosure`: (`counterparty`: *Address*) => *Promise*<string\> ; `fundChannel`: (`me`: *Address*, `counterparty`: *Address*, `myTotal`: *Balance*, `theirTotal`: *Balance*) => *Promise*<string\> ; `getBalance`: (`address`: *Address*) => *Promise*<Balance\> ; `getChannels`: () => *HoprChannels* ; `getGenesisBlock`: () => *number* ; `getInfo`: () => *string* ; `getLatestBlockNumber`: () => *Promise*<number\> ; `getNativeBalance`: (`address`: *Address*) => *Promise*<NativeBalance\> ; `getPrivateKey`: () => *Uint8Array* ; `getPublicKey`: () => *PublicKey* ; `getWallet`: () => *Wallet* ; `initiateChannelClosure`: (`counterparty`: *Address*) => *Promise*<string\> ; `openChannel`: (`me`: *Address*, `counterparty`: *Address*, `amount`: *Balance*) => *Promise*<string\> ; `redeemTicket`: (`counterparty`: *Address*, `ackTicket`: *AcknowledgedTicket*, `ticket`: *Ticket*) => *Promise*<string\> ; `setCommitment`: (`comm`: *Hash*) => *Promise*<string\> ; `subscribeBlock`: (`cb`: *any*) => *JsonRpcProvider* \| *WebSocketProvider* ; `subscribeChannelEvents`: (`cb`: *any*) => *HoprChannels* ; `subscribeError`: (`cb`: *any*) => *void* ; `unsubscribe`: () => *void* ; `waitUntilReady`: () => *Promise*<Network\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: *string*, `amount`: *string*) => *Promise*<string\>  }, `indexer`: [*Indexer*](indexer.md), `privateKey`: *Uint8Array*): [*Channel*](channel.md)

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
| `chain.setCommitment` | (`comm`: *Hash*) => *Promise*<string\> |
| `chain.subscribeBlock` | (`cb`: *any*) => *JsonRpcProvider* \| *WebSocketProvider* |
| `chain.subscribeChannelEvents` | (`cb`: *any*) => *HoprChannels* |
| `chain.subscribeError` | (`cb`: *any*) => *void* |
| `chain.unsubscribe` | () => *void* |
| `chain.waitUntilReady` | () => *Promise*<Network\> |
| `chain.withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: *string*, `amount`: *string*) => *Promise*<string\> |
| `indexer` | [*Indexer*](indexer.md) |
| `privateKey` | *Uint8Array* |

**Returns:** [*Channel*](channel.md)

Defined in: [core-ethereum/src/channel.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L26)

## Properties

### commitment

• `Private` **commitment**: *Commitment*

Defined in: [core-ethereum/src/channel.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L26)

___

### index

• `Private` **index**: *number*

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

Defined in: [core-ethereum/src/channel.ts:54](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L54)

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

Defined in: [core-ethereum/src/channel.ts:185](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L185)

___

### createTicket

▸ **createTicket**(`amount`: *Balance*, `challenge`: *Challenge*, `winProb`: *number*): *Promise*<Ticket\>

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
| `winProb` | *number* | the winning probability to use |

**Returns:** *Promise*<Ticket\>

a signed ticket

Defined in: [core-ethereum/src/channel.ts:164](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L164)

___

### finalizeClosure

▸ **finalizeClosure**(): *Promise*<string\>

**Returns:** *Promise*<string\>

Defined in: [core-ethereum/src/channel.ts:143](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L143)

___

### fund

▸ **fund**(`myFund`: *Balance*, `counterpartyFund`: *Balance*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `myFund` | *Balance* |
| `counterpartyFund` | *Balance* |

**Returns:** *Promise*<void\>

Defined in: [core-ethereum/src/channel.ts:104](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L104)

___

### getBalances

▸ **getBalances**(): *Promise*<{ `counterparty`: *Balance* ; `self`: *Balance*  }\>

**Returns:** *Promise*<{ `counterparty`: *Balance* ; `self`: *Balance*  }\>

Defined in: [core-ethereum/src/channel.ts:92](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L92)

___

### getChainCommitment

▸ **getChainCommitment**(): *Promise*<Hash\>

**Returns:** *Promise*<Hash\>

Defined in: [core-ethereum/src/channel.ts:79](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L79)

___

### getId

▸ **getId**(): *Hash*

**Returns:** *Hash*

Defined in: [core-ethereum/src/channel.ts:75](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L75)

___

### getState

▸ **getState**(): *Promise*<ChannelEntry\>

**Returns:** *Promise*<ChannelEntry\>

Defined in: [core-ethereum/src/channel.ts:83](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L83)

___

### initializeClosure

▸ **initializeClosure**(): *Promise*<string\>

**Returns:** *Promise*<string\>

Defined in: [core-ethereum/src/channel.ts:134](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L134)

___

### open

▸ **open**(`fundAmount`: *Balance*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `fundAmount` | *Balance* |

**Returns:** *Promise*<void\>

Defined in: [core-ethereum/src/channel.ts:115](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L115)

___

### redeemTicket

▸ **redeemTicket**(`ackTicket`: *AcknowledgedTicket*): *Promise*<[*RedeemTicketResponse*](../modules.md#redeemticketresponse)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackTicket` | *AcknowledgedTicket* |

**Returns:** *Promise*<[*RedeemTicketResponse*](../modules.md#redeemticketresponse)\>

Defined in: [core-ethereum/src/channel.ts:199](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L199)

___

### generateId

▸ `Static` **generateId**(`self`: *Address*, `counterparty`: *Address*): *Hash*

#### Parameters

| Name | Type |
| :------ | :------ |
| `self` | *Address* |
| `counterparty` | *Address* |

**Returns:** *Hash*

Defined in: [core-ethereum/src/channel.ts:45](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L45)
