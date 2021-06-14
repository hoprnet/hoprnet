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
- [getAcknowledgedTickets](channel.md#getacknowledgedtickets)
- [getBalances](channel.md#getbalances)
- [getChainCommitment](channel.md#getchaincommitment)
- [initializeClosure](channel.md#initializeclosure)
- [open](channel.md#open)
- [redeemAllTickets](channel.md#redeemalltickets)
- [redeemTicket](channel.md#redeemticket)
- [themToUs](channel.md#themtous)
- [usToThem](channel.md#ustothem)

## Constructors

### constructor

• **new Channel**(`self`, `counterparty`, `db`, `chain`, `indexer`, `privateKey`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `self` | `PublicKey` |
| `counterparty` | `PublicKey` |
| `db` | `HoprDB` |
| `chain` | `Object` |
| `chain.announce` | (`multiaddr`: `Multiaddr`) => `Promise`<string\> |
| `chain.finalizeChannelClosure` | (`counterparty`: `Address`) => `Promise`<string\> |
| `chain.fundChannel` | (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`) => `Promise`<string\> |
| `chain.getBalance` | (`address`: `Address`) => `Promise`<Balance\> |
| `chain.getChannels` | () => `HoprChannels` |
| `chain.getGenesisBlock` | () => `number` |
| `chain.getInfo` | () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` ; `hoprTokenAddress`: `string` ; `network`: `Networks`  } |
| `chain.getLatestBlockNumber` | () => `Promise`<number\> |
| `chain.getNativeBalance` | (`address`: `Address`) => `Promise`<NativeBalance\> |
| `chain.getPrivateKey` | () => `Uint8Array` |
| `chain.getPublicKey` | () => `PublicKey` |
| `chain.getWallet` | () => `Wallet` |
| `chain.initiateChannelClosure` | (`counterparty`: `Address`) => `Promise`<string\> |
| `chain.openChannel` | (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`) => `Promise`<string\> |
| `chain.redeemTicket` | (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`) => `Promise`<string\> |
| `chain.setCommitment` | (`counterparty`: `Address`, `comm`: `Hash`) => `Promise`<string\> |
| `chain.subscribeBlock` | (`cb`: `any`) => `JsonRpcProvider` \| `WebSocketProvider` |
| `chain.subscribeChannelEvents` | (`cb`: `any`) => `HoprChannels` |
| `chain.subscribeError` | (`cb`: `any`) => `void` |
| `chain.unsubscribe` | () => `void` |
| `chain.waitUntilReady` | () => `Promise`<Network\> |
| `chain.withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`) => `Promise`<string\> |
| `indexer` | [Indexer](indexer.md) |
| `privateKey` | `Uint8Array` |

#### Defined in

[core-ethereum/src/channel.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L28)

## Properties

### commitment

• `Private` **commitment**: `Commitment`

#### Defined in

[core-ethereum/src/channel.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L28)

## Methods

### acknowledge

▸ **acknowledge**(`unacknowledgedTicket`, `acknowledgement`): `Promise`<AcknowledgedTicket\>

Reserve a preImage for the given ticket if it is a winning ticket.

#### Parameters

| Name | Type |
| :------ | :------ |
| `unacknowledgedTicket` | `UnacknowledgedTicket` |
| `acknowledgement` | `HalfKey` |

#### Returns

`Promise`<AcknowledgedTicket\>

#### Defined in

[core-ethereum/src/channel.ts:51](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L51)

___

### bumpTicketIndex

▸ `Private` **bumpTicketIndex**(`channelId`): `Promise`<UINT256\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | `Hash` |

#### Returns

`Promise`<UINT256\>

#### Defined in

[core-ethereum/src/channel.ts:150](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L150)

___

### createDummyTicket

▸ **createDummyTicket**(`challenge`): `Ticket`

Creates a ticket that is sent next to the packet to the last node.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `challenge` | `Challenge` | dummy challenge, potential no valid response known |

#### Returns

`Ticket`

a ticket without any value

#### Defined in

[core-ethereum/src/channel.ts:201](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L201)

___

### createTicket

▸ **createTicket**(`amount`, `challenge`, `winProb`): `Promise`<Ticket\>

Creates a signed ticket that includes the given amount of
tokens

**`dev`** Due to a missing feature, namely ECMUL, in Ethereum, the
challenge is given as an Ethereum address because the signature
recovery algorithm is used to perform an EC-point multiplication.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `amount` | `Balance` | value of the ticket |
| `challenge` | `Challenge` | challenge to solve in order to redeem the ticket |
| `winProb` | `BN` | the winning probability to use |

#### Returns

`Promise`<Ticket\>

a signed ticket

#### Defined in

[core-ethereum/src/channel.ts:173](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L173)

___

### finalizeClosure

▸ **finalizeClosure**(): `Promise`<string\>

#### Returns

`Promise`<string\>

#### Defined in

[core-ethereum/src/channel.ts:140](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L140)

___

### fund

▸ **fund**(`myFund`, `counterpartyFund`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `myFund` | `Balance` |
| `counterpartyFund` | `Balance` |

#### Returns

`Promise`<void\>

#### Defined in

[core-ethereum/src/channel.ts:100](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L100)

___

### getAcknowledgedTickets

▸ **getAcknowledgedTickets**(): `Promise`<AcknowledgedTicket[]\>

#### Returns

`Promise`<AcknowledgedTicket[]\>

#### Defined in

[core-ethereum/src/channel.ts:215](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L215)

___

### getBalances

▸ **getBalances**(): `Promise`<Balance[]\>

#### Returns

`Promise`<Balance[]\>

#### Defined in

[core-ethereum/src/channel.ts:96](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L96)

___

### getChainCommitment

▸ **getChainCommitment**(): `Promise`<Hash\>

#### Returns

`Promise`<Hash\>

#### Defined in

[core-ethereum/src/channel.ts:83](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L83)

___

### initializeClosure

▸ **initializeClosure**(): `Promise`<string\>

#### Returns

`Promise`<string\>

#### Defined in

[core-ethereum/src/channel.ts:131](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L131)

___

### open

▸ **open**(`fundAmount`): `Promise`<Hash\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `fundAmount` | `Balance` |

#### Returns

`Promise`<Hash\>

#### Defined in

[core-ethereum/src/channel.ts:111](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L111)

___

### redeemAllTickets

▸ **redeemAllTickets**(): `Promise`<[RedeemTicketResponse](../modules.md#redeemticketresponse)[]\>

#### Returns

`Promise`<[RedeemTicketResponse](../modules.md#redeemticketresponse)[]\>

#### Defined in

[core-ethereum/src/channel.ts:219](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L219)

___

### redeemTicket

▸ **redeemTicket**(`ackTicket`): `Promise`<[RedeemTicketResponse](../modules.md#redeemticketresponse)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackTicket` | `AcknowledgedTicket` |

#### Returns

`Promise`<[RedeemTicketResponse](../modules.md#redeemticketresponse)\>

#### Defined in

[core-ethereum/src/channel.ts:223](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L223)

___

### themToUs

▸ **themToUs**(): `Promise`<ChannelEntry\>

#### Returns

`Promise`<ChannelEntry\>

#### Defined in

[core-ethereum/src/channel.ts:91](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L91)

___

### usToThem

▸ **usToThem**(): `Promise`<ChannelEntry\>

#### Returns

`Promise`<ChannelEntry\>

#### Defined in

[core-ethereum/src/channel.ts:87](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L87)
