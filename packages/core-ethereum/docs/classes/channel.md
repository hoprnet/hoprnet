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

\+ **new Channel**(`self`: *PublicKey*, `counterparty`: *PublicKey*, `db`: *HoprDB*, `chain`: { `announce`: (`multiaddr`: Multiaddr) => *Promise*<string\> ; `finalizeChannelClosure`: (`counterparty`: *any*) => *Promise*<string\> ; `fundChannel`: (`me`: *Address*, `counterparty`: *Address*, `myTotal`: *Balance*, `theirTotal`: *Balance*) => *Promise*<string\> ; `getBalance`: (`address`: *Address*) => *Promise*<Balance\> ; `getChannels`: () => *HoprChannels* ; `getGenesisBlock`: () => *number* ; `getInfo`: () => *string* ; `getLatestBlockNumber`: () => *Promise*<number\> ; `getNativeBalance`: (`address`: *any*) => *Promise*<NativeBalance\> ; `getPrivateKey`: () => *Uint8Array* ; `getPublicKey`: () => *PublicKey* ; `getWallet`: () => *Wallet* ; `initiateChannelClosure`: (`counterparty`: *any*) => *Promise*<string\> ; `openChannel`: (`me`: *any*, `counterparty`: *any*, `amount`: *any*) => *Promise*<string\> ; `redeemTicket`: (`counterparty`: *any*, `ackTicket`: *any*, `ticket`: *any*) => *Promise*<string\> ; `setCommitment`: (`comm`: *Hash*) => *Promise*<string\> ; `subscribeBlock`: (`cb`: *any*) => *JsonRpcProvider* \| *WebSocketProvider* ; `subscribeChannelEvents`: (`cb`: *any*) => *HoprChannels* ; `subscribeError`: (`cb`: *any*) => *void* ; `unsubscribe`: () => *void* ; `waitUntilReady`: () => *Promise*<Network\> ; `withdraw`: (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: *string*, `amount`: *string*) => *Promise*<string\>  }, `indexer`: [*Indexer*](indexer.md), `privateKey`: *Uint8Array*): [*Channel*](channel.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `self` | *PublicKey* |
| `counterparty` | *PublicKey* |
| `db` | *HoprDB* |
| `chain` | *object* |
| `chain.announce` | (`multiaddr`: Multiaddr) => *Promise*<string\> |
| `chain.finalizeChannelClosure` | (`counterparty`: *any*) => *Promise*<string\> |
| `chain.fundChannel` | (`me`: *Address*, `counterparty`: *Address*, `myTotal`: *Balance*, `theirTotal`: *Balance*) => *Promise*<string\> |
| `chain.getBalance` | (`address`: *Address*) => *Promise*<Balance\> |
| `chain.getChannels` | () => *HoprChannels* |
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
| `chain.subscribeChannelEvents` | (`cb`: *any*) => *HoprChannels* |
| `chain.subscribeError` | (`cb`: *any*) => *void* |
| `chain.unsubscribe` | () => *void* |
| `chain.waitUntilReady` | () => *Promise*<Network\> |
| `chain.withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: *string*, `amount`: *string*) => *Promise*<string\> |
| `indexer` | [*Indexer*](indexer.md) |
| `privateKey` | *Uint8Array* |

**Returns:** [*Channel*](channel.md)

Defined in: [core-ethereum/src/channel.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L24)

## Properties

### commitment

• `Private` **commitment**: *Commitment*

Defined in: [core-ethereum/src/channel.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L24)

___

### index

• `Private` **index**: *number*

Defined in: [core-ethereum/src/channel.ts:23](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L23)

## Methods

### acknowledge

▸ **acknowledge**(`unacknowledgedTicket`: *UnacknowledgedTicket*, `acknowledgement`: *Hash*): *Promise*<AcknowledgedTicket\>

Reserve a preImage for the given ticket if it is a winning ticket.

#### Parameters

| Name | Type |
| :------ | :------ |
| `unacknowledgedTicket` | *UnacknowledgedTicket* |
| `acknowledgement` | *Hash* |

**Returns:** *Promise*<AcknowledgedTicket\>

Defined in: [core-ethereum/src/channel.ts:52](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L52)

___

### createDummyTicket

▸ **createDummyTicket**(`challenge`: *PublicKey*): *Ticket*

Creates a ticket that is sent next to the packet to the last node.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `challenge` | *PublicKey* | dummy challenge, potential no valid response known |

**Returns:** *Ticket*

a ticket without any value

Defined in: [core-ethereum/src/channel.ts:192](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L192)

___

### createTicket

▸ **createTicket**(`amount`: *Balance*, `challenge`: *PublicKey*, `winProb`: *number*): *Promise*<Ticket\>

Creates a signed ticket that includes the given amount of
tokens

**`dev`** Due to a missing feature, namely ECMUL, in Ethereum, the
challenge is given as an Ethereum address because the signature
recovery algorithm is used to perform an EC-point multiplication.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `amount` | *Balance* | value of the ticket |
| `challenge` | *PublicKey* | challenge to solve in order to redeem the ticket |
| `winProb` | *number* | the winning probability to use |

**Returns:** *Promise*<Ticket\>

a signed ticket

Defined in: [core-ethereum/src/channel.ts:171](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L171)

___

### finalizeClosure

▸ **finalizeClosure**(): *Promise*<string\>

**Returns:** *Promise*<string\>

Defined in: [core-ethereum/src/channel.ts:150](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L150)

___

### fund

▸ **fund**(`myFund`: *Balance*, `counterpartyFund`: *Balance*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `myFund` | *Balance* |
| `counterpartyFund` | *Balance* |

**Returns:** *Promise*<void\>

Defined in: [core-ethereum/src/channel.ts:111](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L111)

___

### getBalances

▸ **getBalances**(): *Promise*<{ `counterparty`: *Balance* ; `self`: *Balance*  }\>

**Returns:** *Promise*<{ `counterparty`: *Balance* ; `self`: *Balance*  }\>

Defined in: [core-ethereum/src/channel.ts:99](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L99)

___

### getChainCommitment

▸ **getChainCommitment**(): *Promise*<Hash\>

**Returns:** *Promise*<Hash\>

Defined in: [core-ethereum/src/channel.ts:86](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L86)

___

### getId

▸ **getId**(): *Hash*

**Returns:** *Hash*

Defined in: [core-ethereum/src/channel.ts:82](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L82)

___

### getState

▸ **getState**(): *Promise*<ChannelEntry\>

**Returns:** *Promise*<ChannelEntry\>

Defined in: [core-ethereum/src/channel.ts:90](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L90)

___

### initializeClosure

▸ **initializeClosure**(): *Promise*<string\>

**Returns:** *Promise*<string\>

Defined in: [core-ethereum/src/channel.ts:141](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L141)

___

### open

▸ **open**(`fundAmount`: *Balance*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `fundAmount` | *Balance* |

**Returns:** *Promise*<void\>

Defined in: [core-ethereum/src/channel.ts:122](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L122)

___

### redeemTicket

▸ **redeemTicket**(`ackTicket`: *AcknowledgedTicket*): *Promise*<[*RedeemTicketResponse*](../modules.md#redeemticketresponse)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackTicket` | *AcknowledgedTicket* |

**Returns:** *Promise*<[*RedeemTicketResponse*](../modules.md#redeemticketresponse)\>

Defined in: [core-ethereum/src/channel.ts:206](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L206)

___

### generateId

▸ `Static` **generateId**(`self`: *Address*, `counterparty`: *Address*): *Hash*

#### Parameters

| Name | Type |
| :------ | :------ |
| `self` | *Address* |
| `counterparty` | *Address* |

**Returns:** *Hash*

Defined in: [core-ethereum/src/channel.ts:43](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L43)
