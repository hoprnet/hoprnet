[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / Channel

# Class: Channel

## Table of contents

### Constructors

- [constructor](channel.md#constructor)

### Properties

- [commitment](channel.md#commitment)

### Methods

- [acknowledge](channel.md#acknowledge)
- [balanceToThem](channel.md#balancetothem)
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

• **new Channel**(`self`, `counterparty`, `db`, `chain`, `indexer`, `privateKey`, `events`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `self` | `PublicKey` |
| `counterparty` | `PublicKey` |
| `db` | `HoprDB` |
| `chain` | `Object` |
| `chain.announce` | (`multiaddr`: `Multiaddr`) => `Promise`<`string`\> |
| `chain.finalizeChannelClosure` | (`counterparty`: `Address`) => `Promise`<`string`\> |
| `chain.fundChannel` | (`me`: `Address`, `counterparty`: `Address`, `myTotal`: `Balance`, `theirTotal`: `Balance`) => `Promise`<`string`\> |
| `chain.getBalance` | (`address`: `Address`) => `Promise`<`Balance`\> |
| `chain.getChannels` | () => `HoprChannels` |
| `chain.getGenesisBlock` | () => `number` |
| `chain.getInfo` | () => { `channelClosureSecs`: `number` ; `hoprChannelsAddress`: `string` ; `hoprTokenAddress`: `string` ; `network`: `Networks`  } |
| `chain.getLatestBlockNumber` | () => `Promise`<`number`\> |
| `chain.getNativeBalance` | (`address`: `Address`) => `Promise`<`NativeBalance`\> |
| `chain.getPrivateKey` | () => `Uint8Array` |
| `chain.getPublicKey` | () => `PublicKey` |
| `chain.getWallet` | () => `Wallet` |
| `chain.initiateChannelClosure` | (`counterparty`: `Address`) => `Promise`<`string`\> |
| `chain.openChannel` | (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`) => `Promise`<`string`\> |
| `chain.redeemTicket` | (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`) => `Promise`<`string`\> |
| `chain.setCommitment` | (`counterparty`: `Address`, `comm`: `Hash`) => `Promise`<`string`\> |
| `chain.subscribeBlock` | (`cb`: `any`) => `JsonRpcProvider` \| `WebSocketProvider` |
| `chain.subscribeChannelEvents` | (`cb`: `any`) => `HoprChannels` |
| `chain.subscribeError` | (`cb`: `any`) => `void` |
| `chain.unsubscribe` | () => `void` |
| `chain.waitUntilReady` | () => `Promise`<`Network`\> |
| `chain.withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`) => `Promise`<`string`\> |
| `indexer` | [`Indexer`](indexer.md) |
| `privateKey` | `Uint8Array` |
| `events` | `EventEmitter` |

#### Defined in

[core-ethereum/src/channel.ts:31](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L31)

## Properties

### commitment

• `Private` **commitment**: `Commitment`

#### Defined in

[core-ethereum/src/channel.ts:31](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L31)

## Methods

### acknowledge

▸ **acknowledge**(`unacknowledgedTicket`, `acknowledgement`): `Promise`<`AcknowledgedTicket`\>

Reserve a preImage for the given ticket if it is a winning ticket.

#### Parameters

| Name | Type |
| :------ | :------ |
| `unacknowledgedTicket` | `UnacknowledgedTicket` |
| `acknowledgement` | `HalfKey` |

#### Returns

`Promise`<`AcknowledgedTicket`\>

#### Defined in

[core-ethereum/src/channel.ts:55](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L55)

___

### balanceToThem

▸ **balanceToThem**(): `Promise`<`Object`\>

#### Returns

`Promise`<`Object`\>

#### Defined in

[core-ethereum/src/channel.ts:228](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L228)

___

### bumpTicketIndex

▸ `Private` **bumpTicketIndex**(`channelId`): `Promise`<`UINT256`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | `Hash` |

#### Returns

`Promise`<`UINT256`\>

#### Defined in

[core-ethereum/src/channel.ts:155](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L155)

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

[core-ethereum/src/channel.ts:209](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L209)

___

### createTicket

▸ **createTicket**(`pathLength`, `challenge`): `Promise`<`Ticket`\>

Creates a signed ticket that includes the given amount of
tokens

**`dev`** Due to a missing feature, namely ECMUL, in Ethereum, the
challenge is given as an Ethereum address because the signature
recovery algorithm is used to perform an EC-point multiplication.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `pathLength` | `number` | - |
| `challenge` | `Challenge` | challenge to solve in order to redeem the ticket |

#### Returns

`Promise`<`Ticket`\>

a signed ticket

#### Defined in

[core-ethereum/src/channel.ts:178](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L178)

___

### finalizeClosure

▸ **finalizeClosure**(): `Promise`<`string`\>

#### Returns

`Promise`<`string`\>

#### Defined in

[core-ethereum/src/channel.ts:145](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L145)

___

### fund

▸ **fund**(`myFund`, `counterpartyFund`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `myFund` | `Balance` |
| `counterpartyFund` | `Balance` |

#### Returns

`Promise`<`void`\>

#### Defined in

[core-ethereum/src/channel.ts:105](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L105)

___

### getAcknowledgedTickets

▸ **getAcknowledgedTickets**(): `Promise`<`AcknowledgedTicket`[]\>

#### Returns

`Promise`<`AcknowledgedTicket`[]\>

#### Defined in

[core-ethereum/src/channel.ts:238](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L238)

___

### getBalances

▸ **getBalances**(): `Promise`<`Balance`[]\>

#### Returns

`Promise`<`Balance`[]\>

#### Defined in

[core-ethereum/src/channel.ts:101](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L101)

___

### getChainCommitment

▸ **getChainCommitment**(): `Promise`<`Hash`\>

#### Returns

`Promise`<`Hash`\>

#### Defined in

[core-ethereum/src/channel.ts:88](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L88)

___

### initializeClosure

▸ **initializeClosure**(): `Promise`<`string`\>

#### Returns

`Promise`<`string`\>

#### Defined in

[core-ethereum/src/channel.ts:136](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L136)

___

### open

▸ **open**(`fundAmount`): `Promise`<`Hash`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `fundAmount` | `Balance` |

#### Returns

`Promise`<`Hash`\>

#### Defined in

[core-ethereum/src/channel.ts:116](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L116)

___

### redeemAllTickets

▸ **redeemAllTickets**(): `Promise`<[`RedeemTicketResponse`](../modules.md#redeemticketresponse)[]\>

#### Returns

`Promise`<[`RedeemTicketResponse`](../modules.md#redeemticketresponse)[]\>

#### Defined in

[core-ethereum/src/channel.ts:242](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L242)

___

### redeemTicket

▸ **redeemTicket**(`ackTicket`): `Promise`<[`RedeemTicketResponse`](../modules.md#redeemticketresponse)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackTicket` | `AcknowledgedTicket` |

#### Returns

`Promise`<[`RedeemTicketResponse`](../modules.md#redeemticketresponse)\>

#### Defined in

[core-ethereum/src/channel.ts:246](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L246)

___

### themToUs

▸ **themToUs**(): `Promise`<[`ChannelEntry`](channelentry.md)\>

#### Returns

`Promise`<[`ChannelEntry`](channelentry.md)\>

#### Defined in

[core-ethereum/src/channel.ts:96](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L96)

___

### usToThem

▸ **usToThem**(): `Promise`<[`ChannelEntry`](channelentry.md)\>

#### Returns

`Promise`<[`ChannelEntry`](channelentry.md)\>

#### Defined in

[core-ethereum/src/channel.ts:92](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L92)
