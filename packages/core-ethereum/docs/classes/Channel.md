[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / Channel

# Class: Channel

## Table of contents

### Constructors

- [constructor](Channel.md#constructor)

### Methods

- [acknowledge](Channel.md#acknowledge)
- [balanceToThem](Channel.md#balancetothem)
- [bumpTicketIndex](Channel.md#bumpticketindex)
- [createDummyTicket](Channel.md#createdummyticket)
- [createTicket](Channel.md#createticket)
- [finalizeClosure](Channel.md#finalizeclosure)
- [fund](Channel.md#fund)
- [getAcknowledgedTickets](Channel.md#getacknowledgedtickets)
- [getChainCommitment](Channel.md#getchaincommitment)
- [getThemToUsId](Channel.md#getthemtousid)
- [getUsToThemId](Channel.md#getustothemid)
- [initializeClosure](Channel.md#initializeclosure)
- [open](Channel.md#open)
- [redeemAllTickets](Channel.md#redeemalltickets)
- [redeemTicket](Channel.md#redeemticket)
- [themToUs](Channel.md#themtous)
- [usToThem](Channel.md#ustothem)

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
| `chain.getNativeTokenTransactionInBlock` | (`blockNumber`: `number`, `isOutgoing`: `boolean`) => `Promise`<`string`[]\> |
| `chain.getPrivateKey` | () => `Uint8Array` |
| `chain.getPublicKey` | () => `PublicKey` |
| `chain.getWallet` | () => `Wallet` |
| `chain.initiateChannelClosure` | (`counterparty`: `Address`) => `Promise`<`string`\> |
| `chain.openChannel` | (`me`: `Address`, `counterparty`: `Address`, `amount`: `Balance`) => `Promise`<`string`\> |
| `chain.redeemTicket` | (`counterparty`: `Address`, `ackTicket`: `AcknowledgedTicket`, `ticket`: `Ticket`) => `Promise`<`string`\> |
| `chain.setCommitment` | (`counterparty`: `Address`, `comm`: `Hash`) => `Promise`<`string`\> |
| `chain.subscribeBlock` | (`cb`: `any`) => `StaticJsonRpcProvider` \| `WebSocketProvider` |
| `chain.subscribeChannelEvents` | (`cb`: `any`) => `HoprChannels` |
| `chain.subscribeError` | (`cb`: `any`) => `void` |
| `chain.subscribeTokenEvents` | (`cb`: `any`) => `HoprToken` |
| `chain.unsubscribe` | () => `void` |
| `chain.updateConfirmedTransaction` | (`hash`: `string`) => `void` |
| `chain.waitUntilReady` | () => `Promise`<`Network`\> |
| `chain.withdraw` | (`currency`: ``"NATIVE"`` \| ``"HOPR"``, `recipient`: `string`, `amount`: `string`) => `Promise`<`string`\> |
| `indexer` | [`Indexer`](Indexer.md) |
| `privateKey` | `Uint8Array` |
| `events` | `EventEmitter` |

#### Defined in

[packages/core-ethereum/src/channel.ts:32](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L32)

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

[packages/core-ethereum/src/channel.ts:46](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L46)

___

### balanceToThem

▸ **balanceToThem**(): `Promise`<`Object`\>

#### Returns

`Promise`<`Object`\>

#### Defined in

[packages/core-ethereum/src/channel.ts:242](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L242)

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

[packages/core-ethereum/src/channel.ts:169](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L169)

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

[packages/core-ethereum/src/channel.ts:223](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L223)

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

[packages/core-ethereum/src/channel.ts:192](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L192)

___

### finalizeClosure

▸ **finalizeClosure**(): `Promise`<`string`\>

#### Returns

`Promise`<`string`\>

#### Defined in

[packages/core-ethereum/src/channel.ts:159](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L159)

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

[packages/core-ethereum/src/channel.ts:119](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L119)

___

### getAcknowledgedTickets

▸ **getAcknowledgedTickets**(): `Promise`<`AcknowledgedTicket`[]\>

#### Returns

`Promise`<`AcknowledgedTicket`[]\>

#### Defined in

[packages/core-ethereum/src/channel.ts:252](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L252)

___

### getChainCommitment

▸ **getChainCommitment**(): `Promise`<`Hash`\>

#### Returns

`Promise`<`Hash`\>

#### Defined in

[packages/core-ethereum/src/channel.ts:99](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L99)

___

### getThemToUsId

▸ **getThemToUsId**(): `Hash`

#### Returns

`Hash`

#### Defined in

[packages/core-ethereum/src/channel.ts:111](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L111)

___

### getUsToThemId

▸ **getUsToThemId**(): `Hash`

#### Returns

`Hash`

#### Defined in

[packages/core-ethereum/src/channel.ts:103](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L103)

___

### initializeClosure

▸ **initializeClosure**(): `Promise`<`string`\>

#### Returns

`Promise`<`string`\>

#### Defined in

[packages/core-ethereum/src/channel.ts:150](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L150)

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

[packages/core-ethereum/src/channel.ts:130](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L130)

___

### redeemAllTickets

▸ **redeemAllTickets**(): `Promise`<[`RedeemTicketResponse`](../modules.md#redeemticketresponse)[]\>

#### Returns

`Promise`<[`RedeemTicketResponse`](../modules.md#redeemticketresponse)[]\>

#### Defined in

[packages/core-ethereum/src/channel.ts:256](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L256)

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

[packages/core-ethereum/src/channel.ts:260](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L260)

___

### themToUs

▸ **themToUs**(): `Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Defined in

[packages/core-ethereum/src/channel.ts:115](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L115)

___

### usToThem

▸ **usToThem**(): `Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Defined in

[packages/core-ethereum/src/channel.ts:107](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/channel.ts#L107)
