[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [channel](../modules/channel.md) / Channel

# Class: Channel

[channel](../modules/channel.md).Channel

## Table of contents

### Constructors

- [constructor](channel.channel-1.md#constructor)

### Properties

- [commitment](channel.channel-1.md#commitment)
- [index](channel.channel-1.md#index)

### Methods

- [acknowledge](channel.channel-1.md#acknowledge)
- [createDummyTicket](channel.channel-1.md#createdummyticket)
- [createTicket](channel.channel-1.md#createticket)
- [finalizeClosure](channel.channel-1.md#finalizeclosure)
- [fund](channel.channel-1.md#fund)
- [getBalances](channel.channel-1.md#getbalances)
- [getChainCommitment](channel.channel-1.md#getchaincommitment)
- [getId](channel.channel-1.md#getid)
- [getState](channel.channel-1.md#getstate)
- [initializeClosure](channel.channel-1.md#initializeclosure)
- [open](channel.channel-1.md#open)
- [redeemTicket](channel.channel-1.md#redeemticket)
- [generateId](channel.channel-1.md#generateid)

## Constructors

### constructor

\+ **new Channel**(`self`: _PublicKey_, `counterparty`: _PublicKey_, `db`: _HoprDB_, `chain`: { `announce`: (`multiaddr`: Multiaddr) => _Promise_<string\> ; `finalizeChannelClosure`: (`counterparty`: _any_) => _Promise_<string\> ; `fundChannel`: (`me`: _Address_, `counterparty`: _Address_, `myTotal`: _Balance_, `theirTotal`: _Balance_) => _Promise_<string\> ; `getBalance`: (`address`: _Address_) => _Promise_<Balance\> ; `getChannels`: () => [_HoprChannels_](contracts_hoprchannels.hoprchannels.md) ; `getGenesisBlock`: () => _number_ ; `getInfo`: () => _string_ ; `getLatestBlockNumber`: () => _Promise_<number\> ; `getNativeBalance`: (`address`: _any_) => _Promise_<NativeBalance\> ; `getPrivateKey`: () => _Uint8Array_ ; `getPublicKey`: () => _PublicKey_ ; `getWallet`: () => _Wallet_ ; `initiateChannelClosure`: (`counterparty`: _any_) => _Promise_<string\> ; `openChannel`: (`me`: _any_, `counterparty`: _any_, `amount`: _any_) => _Promise_<string\> ; `redeemTicket`: (`counterparty`: _any_, `ackTicket`: _any_, `ticket`: _any_) => _Promise_<string\> ; `setCommitment`: (`comm`: _Hash_) => _Promise_<string\> ; `subscribeBlock`: (`cb`: _any_) => _JsonRpcProvider_ \| _WebSocketProvider_ ; `subscribeChannelEvents`: (`cb`: _any_) => [_HoprChannels_](contracts_hoprchannels.hoprchannels.md) ; `subscribeError`: (`cb`: _any_) => _void_ ; `unsubscribe`: () => _void_ ; `waitUntilReady`: () => _Promise_<Network\> ; `withdraw`: (`currency`: `"NATIVE"` \| `"HOPR"`, `recipient`: _string_, `amount`: _string_) => _Promise_<string\> }, `indexer`: [_default_](indexer.default.md), `privateKey`: _Uint8Array_): [_Channel_](channel.channel-1.md)

#### Parameters

| Name                           | Type                                                                                                              |
| :----------------------------- | :---------------------------------------------------------------------------------------------------------------- |
| `self`                         | _PublicKey_                                                                                                       |
| `counterparty`                 | _PublicKey_                                                                                                       |
| `db`                           | _HoprDB_                                                                                                          |
| `chain`                        | _object_                                                                                                          |
| `chain.announce`               | (`multiaddr`: Multiaddr) => _Promise_<string\>                                                                    |
| `chain.finalizeChannelClosure` | (`counterparty`: _any_) => _Promise_<string\>                                                                     |
| `chain.fundChannel`            | (`me`: _Address_, `counterparty`: _Address_, `myTotal`: _Balance_, `theirTotal`: _Balance_) => _Promise_<string\> |
| `chain.getBalance`             | (`address`: _Address_) => _Promise_<Balance\>                                                                     |
| `chain.getChannels`            | () => [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)                                                    |
| `chain.getGenesisBlock`        | () => _number_                                                                                                    |
| `chain.getInfo`                | () => _string_                                                                                                    |
| `chain.getLatestBlockNumber`   | () => _Promise_<number\>                                                                                          |
| `chain.getNativeBalance`       | (`address`: _any_) => _Promise_<NativeBalance\>                                                                   |
| `chain.getPrivateKey`          | () => _Uint8Array_                                                                                                |
| `chain.getPublicKey`           | () => _PublicKey_                                                                                                 |
| `chain.getWallet`              | () => _Wallet_                                                                                                    |
| `chain.initiateChannelClosure` | (`counterparty`: _any_) => _Promise_<string\>                                                                     |
| `chain.openChannel`            | (`me`: _any_, `counterparty`: _any_, `amount`: _any_) => _Promise_<string\>                                       |
| `chain.redeemTicket`           | (`counterparty`: _any_, `ackTicket`: _any_, `ticket`: _any_) => _Promise_<string\>                                |
| `chain.setCommitment`          | (`comm`: _Hash_) => _Promise_<string\>                                                                            |
| `chain.subscribeBlock`         | (`cb`: _any_) => _JsonRpcProvider_ \| _WebSocketProvider_                                                         |
| `chain.subscribeChannelEvents` | (`cb`: _any_) => [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)                                         |
| `chain.subscribeError`         | (`cb`: _any_) => _void_                                                                                           |
| `chain.unsubscribe`            | () => _void_                                                                                                      |
| `chain.waitUntilReady`         | () => _Promise_<Network\>                                                                                         |
| `chain.withdraw`               | (`currency`: `"NATIVE"` \| `"HOPR"`, `recipient`: _string_, `amount`: _string_) => _Promise_<string\>             |
| `indexer`                      | [_default_](indexer.default.md)                                                                                   |
| `privateKey`                   | _Uint8Array_                                                                                                      |

**Returns:** [_Channel_](channel.channel-1.md)

Defined in: [packages/core-ethereum/src/channel.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/channel.ts#L24)

## Properties

### commitment

• `Private` **commitment**: [_Commitment_](commitment.commitment-1.md)

Defined in: [packages/core-ethereum/src/channel.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/channel.ts#L24)

---

### index

• `Private` **index**: _number_

Defined in: [packages/core-ethereum/src/channel.ts:23](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/channel.ts#L23)

## Methods

### acknowledge

▸ **acknowledge**(`unacknowledgedTicket`: _UnacknowledgedTicket_, `acknowledgement`: _Hash_): _Promise_<AcknowledgedTicket\>

Reserve a preImage for the given ticket if it is a winning ticket.

#### Parameters

| Name                   | Type                   |
| :--------------------- | :--------------------- |
| `unacknowledgedTicket` | _UnacknowledgedTicket_ |
| `acknowledgement`      | _Hash_                 |

**Returns:** _Promise_<AcknowledgedTicket\>

Defined in: [packages/core-ethereum/src/channel.ts:52](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/channel.ts#L52)

---

### createDummyTicket

▸ **createDummyTicket**(`challenge`: _PublicKey_): _Ticket_

Creates a ticket that is sent next to the packet to the last node.

#### Parameters

| Name        | Type        | Description                                        |
| :---------- | :---------- | :------------------------------------------------- |
| `challenge` | _PublicKey_ | dummy challenge, potential no valid response known |

**Returns:** _Ticket_

a ticket without any value

Defined in: [packages/core-ethereum/src/channel.ts:192](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/channel.ts#L192)

---

### createTicket

▸ **createTicket**(`amount`: _Balance_, `challenge`: _PublicKey_, `winProb`: _number_): _Promise_<Ticket\>

Creates a signed ticket that includes the given amount of
tokens

**`dev`** Due to a missing feature, namely ECMUL, in Ethereum, the
challenge is given as an Ethereum address because the signature
recovery algorithm is used to perform an EC-point multiplication.

#### Parameters

| Name        | Type        | Description                                      |
| :---------- | :---------- | :----------------------------------------------- |
| `amount`    | _Balance_   | value of the ticket                              |
| `challenge` | _PublicKey_ | challenge to solve in order to redeem the ticket |
| `winProb`   | _number_    | the winning probability to use                   |

**Returns:** _Promise_<Ticket\>

a signed ticket

Defined in: [packages/core-ethereum/src/channel.ts:171](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/channel.ts#L171)

---

### finalizeClosure

▸ **finalizeClosure**(): _Promise_<string\>

**Returns:** _Promise_<string\>

Defined in: [packages/core-ethereum/src/channel.ts:150](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/channel.ts#L150)

---

### fund

▸ **fund**(`myFund`: _Balance_, `counterpartyFund`: _Balance_): _Promise_<void\>

#### Parameters

| Name               | Type      |
| :----------------- | :-------- |
| `myFund`           | _Balance_ |
| `counterpartyFund` | _Balance_ |

**Returns:** _Promise_<void\>

Defined in: [packages/core-ethereum/src/channel.ts:111](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/channel.ts#L111)

---

### getBalances

▸ **getBalances**(): _Promise_<{ `counterparty`: _Balance_ ; `self`: _Balance_ }\>

**Returns:** _Promise_<{ `counterparty`: _Balance_ ; `self`: _Balance_ }\>

Defined in: [packages/core-ethereum/src/channel.ts:99](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/channel.ts#L99)

---

### getChainCommitment

▸ **getChainCommitment**(): _Promise_<Hash\>

**Returns:** _Promise_<Hash\>

Defined in: [packages/core-ethereum/src/channel.ts:86](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/channel.ts#L86)

---

### getId

▸ **getId**(): _Hash_

**Returns:** _Hash_

Defined in: [packages/core-ethereum/src/channel.ts:82](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/channel.ts#L82)

---

### getState

▸ **getState**(): _Promise_<ChannelEntry\>

**Returns:** _Promise_<ChannelEntry\>

Defined in: [packages/core-ethereum/src/channel.ts:90](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/channel.ts#L90)

---

### initializeClosure

▸ **initializeClosure**(): _Promise_<string\>

**Returns:** _Promise_<string\>

Defined in: [packages/core-ethereum/src/channel.ts:141](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/channel.ts#L141)

---

### open

▸ **open**(`fundAmount`: _Balance_): _Promise_<void\>

#### Parameters

| Name         | Type      |
| :----------- | :-------- |
| `fundAmount` | _Balance_ |

**Returns:** _Promise_<void\>

Defined in: [packages/core-ethereum/src/channel.ts:122](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/channel.ts#L122)

---

### redeemTicket

▸ **redeemTicket**(`ackTicket`: _AcknowledgedTicket_): _Promise_<[_RedeemTicketResponse_](../modules/index.md#redeemticketresponse)\>

#### Parameters

| Name        | Type                 |
| :---------- | :------------------- |
| `ackTicket` | _AcknowledgedTicket_ |

**Returns:** _Promise_<[_RedeemTicketResponse_](../modules/index.md#redeemticketresponse)\>

Defined in: [packages/core-ethereum/src/channel.ts:206](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/channel.ts#L206)

---

### generateId

▸ `Static` **generateId**(`self`: _Address_, `counterparty`: _Address_): _Hash_

#### Parameters

| Name           | Type      |
| :------------- | :-------- |
| `self`         | _Address_ |
| `counterparty` | _Address_ |

**Returns:** _Hash_

Defined in: [packages/core-ethereum/src/channel.ts:43](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/channel.ts#L43)
