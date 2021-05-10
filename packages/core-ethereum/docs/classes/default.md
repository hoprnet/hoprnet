[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / default

# Class: default

## Table of contents

### Constructors

- [constructor](default.md#constructor)

### Properties

- [CHAIN_NAME](default.md#chain_name)
- [cachedGetBalance](default.md#cachedgetbalance)
- [cachedGetNativeBalance](default.md#cachedgetnativebalance)
- [indexer](default.md#indexer)
- [privateKey](default.md#privatekey)

### Methods

- [announce](default.md#announce)
- [getAccount](default.md#getaccount)
- [getAddress](default.md#getaddress)
- [getBalance](default.md#getbalance)
- [getChannel](default.md#getchannel)
- [getChannelsFromPeer](default.md#getchannelsfrompeer)
- [getChannelsOf](default.md#getchannelsof)
- [getNativeBalance](default.md#getnativebalance)
- [getPublicKey](default.md#getpublickey)
- [getPublicKeyOf](default.md#getpublickeyof)
- [getRandomChannel](default.md#getrandomchannel)
- [smartContractInfo](default.md#smartcontractinfo)
- [stop](default.md#stop)
- [uncachedGetBalance](default.md#uncachedgetbalance)
- [uncachedGetNativeBalance](default.md#uncachedgetnativebalance)
- [waitForPublicNodes](default.md#waitforpublicnodes)
- [withdraw](default.md#withdraw)
- [create](default.md#create)

## Constructors

### constructor

\+ **new default**(`chain`: { `announce`: (`multiaddr`: Multiaddr) => _Promise_<string\> ; `finalizeChannelClosure`: (`counterparty`: _any_) => _Promise_<string\> ; `fundChannel`: (`me`: _Address_, `counterparty`: _Address_, `myTotal`: _Balance_, `theirTotal`: _Balance_) => _Promise_<string\> ; `getBalance`: (`address`: _Address_) => _Promise_<Balance\> ; `getChannels`: () => _HoprChannels_ ; `getGenesisBlock`: () => _number_ ; `getInfo`: () => _string_ ; `getLatestBlockNumber`: () => _Promise_<number\> ; `getNativeBalance`: (`address`: _any_) => _Promise_<NativeBalance\> ; `getPrivateKey`: () => _Uint8Array_ ; `getPublicKey`: () => _PublicKey_ ; `getWallet`: () => _Wallet_ ; `initiateChannelClosure`: (`counterparty`: _any_) => _Promise_<string\> ; `openChannel`: (`me`: _any_, `counterparty`: _any_, `amount`: _any_) => _Promise_<string\> ; `redeemTicket`: (`counterparty`: _any_, `ackTicket`: _any_, `ticket`: _any_) => _Promise_<string\> ; `setCommitment`: (`comm`: _Hash_) => _Promise_<string\> ; `subscribeBlock`: (`cb`: _any_) => _JsonRpcProvider_ \| _WebSocketProvider_ ; `subscribeChannelEvents`: (`cb`: _any_) => _HoprChannels_ ; `subscribeError`: (`cb`: _any_) => _void_ ; `unsubscribe`: () => _void_ ; `waitUntilReady`: () => _Promise_<Network\> ; `withdraw`: (`currency`: `"NATIVE"` \| `"HOPR"`, `recipient`: _string_, `amount`: _string_) => _Promise_<string\> }, `db`: _HoprDB_, `indexer`: [_Indexer_](indexer.md)): [_default_](default.md)

#### Parameters

| Name                           | Type                                                                                                              |
| :----------------------------- | :---------------------------------------------------------------------------------------------------------------- |
| `chain`                        | _object_                                                                                                          |
| `chain.announce`               | (`multiaddr`: Multiaddr) => _Promise_<string\>                                                                    |
| `chain.finalizeChannelClosure` | (`counterparty`: _any_) => _Promise_<string\>                                                                     |
| `chain.fundChannel`            | (`me`: _Address_, `counterparty`: _Address_, `myTotal`: _Balance_, `theirTotal`: _Balance_) => _Promise_<string\> |
| `chain.getBalance`             | (`address`: _Address_) => _Promise_<Balance\>                                                                     |
| `chain.getChannels`            | () => _HoprChannels_                                                                                              |
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
| `chain.subscribeChannelEvents` | (`cb`: _any_) => _HoprChannels_                                                                                   |
| `chain.subscribeError`         | (`cb`: _any_) => _void_                                                                                           |
| `chain.unsubscribe`            | () => _void_                                                                                                      |
| `chain.waitUntilReady`         | () => _Promise_<Network\>                                                                                         |
| `chain.withdraw`               | (`currency`: `"NATIVE"` \| `"HOPR"`, `recipient`: _string_, `amount`: _string_) => _Promise_<string\>             |
| `db`                           | _HoprDB_                                                                                                          |
| `indexer`                      | [_Indexer_](indexer.md)                                                                                           |

**Returns:** [_default_](default.md)

Defined in: [core-ethereum/src/index.ts:33](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L33)

## Properties

### CHAIN_NAME

• `Readonly` **CHAIN_NAME**: `"HOPR on Ethereum"`= 'HOPR on Ethereum'

Defined in: [core-ethereum/src/index.ts:39](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L39)

---

### cachedGetBalance

• `Private` **cachedGetBalance**: () => _Promise_<Balance\>

#### Type declaration

▸ (): _Promise_<Balance\>

**Returns:** _Promise_<Balance\>

Defined in: utils/lib/cache.d.ts:1

Defined in: [core-ethereum/src/index.ts:82](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L82)

---

### cachedGetNativeBalance

• `Private` **cachedGetNativeBalance**: () => _Promise_<NativeBalance\>

#### Type declaration

▸ (): _Promise_<NativeBalance\>

**Returns:** _Promise_<NativeBalance\>

Defined in: utils/lib/cache.d.ts:1

Defined in: [core-ethereum/src/index.ts:104](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L104)

---

### indexer

• **indexer**: [_Indexer_](indexer.md)

---

### privateKey

• `Private` **privateKey**: _Uint8Array_

Defined in: [core-ethereum/src/index.ts:33](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L33)

## Methods

### announce

▸ **announce**(`multiaddr`: _Multiaddr_): _Promise_<string\>

#### Parameters

| Name        | Type        |
| :---------- | :---------- |
| `multiaddr` | _Multiaddr_ |

**Returns:** _Promise_<string\>

Defined in: [core-ethereum/src/index.ts:53](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L53)

---

### getAccount

▸ **getAccount**(`addr`: _Address_): _Promise_<AccountEntry\>

#### Parameters

| Name   | Type      |
| :----- | :-------- |
| `addr` | _Address_ |

**Returns:** _Promise_<AccountEntry\>

Defined in: [core-ethereum/src/index.ts:69](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L69)

---

### getAddress

▸ **getAddress**(): _Address_

**Returns:** _Address_

Defined in: [core-ethereum/src/index.ts:91](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L91)

---

### getBalance

▸ **getBalance**(`useCache?`: _boolean_): _Promise_<Balance\>

Retrieves HOPR balance, optionally uses the cache.

#### Parameters

| Name       | Type      | Default value |
| :--------- | :-------- | :------------ |
| `useCache` | _boolean_ | false         |

**Returns:** _Promise_<Balance\>

HOPR balance

Defined in: [core-ethereum/src/index.ts:87](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L87)

---

### getChannel

▸ **getChannel**(`src`: _PublicKey_, `counterparty`: _PublicKey_): [_Channel_](channel.md)

#### Parameters

| Name           | Type        |
| :------------- | :---------- |
| `src`          | _PublicKey_ |
| `counterparty` | _PublicKey_ |

**Returns:** [_Channel_](channel.md)

Defined in: [core-ethereum/src/index.ts:49](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L49)

---

### getChannelsFromPeer

▸ **getChannelsFromPeer**(`p`: _PeerId_): _Promise_<[_RoutingChannel_](../modules.md#routingchannel)[]\>

#### Parameters

| Name | Type     |
| :--- | :------- |
| `p`  | _PeerId_ |

**Returns:** _Promise_<[_RoutingChannel_](../modules.md#routingchannel)[]\>

Defined in: [core-ethereum/src/index.ts:61](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L61)

---

### getChannelsOf

▸ **getChannelsOf**(`addr`: _Address_): _Promise_<ChannelEntry[]\>

#### Parameters

| Name   | Type      |
| :----- | :-------- |
| `addr` | _Address_ |

**Returns:** _Promise_<ChannelEntry[]\>

Defined in: [core-ethereum/src/index.ts:65](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L65)

---

### getNativeBalance

▸ **getNativeBalance**(`useCache?`: _boolean_): _Promise_<NativeBalance\>

#### Parameters

| Name       | Type      | Default value |
| :--------- | :-------- | :------------ |
| `useCache` | _boolean_ | false         |

**Returns:** _Promise_<NativeBalance\>

Defined in: [core-ethereum/src/index.ts:108](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L108)

---

### getPublicKey

▸ **getPublicKey**(): _PublicKey_

**Returns:** _PublicKey_

Defined in: [core-ethereum/src/index.ts:95](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L95)

---

### getPublicKeyOf

▸ **getPublicKeyOf**(`addr`: _Address_): _Promise_<PublicKey\>

#### Parameters

| Name   | Type      |
| :----- | :-------- |
| `addr` | _Address_ |

**Returns:** _Promise_<PublicKey\>

Defined in: [core-ethereum/src/index.ts:73](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L73)

---

### getRandomChannel

▸ **getRandomChannel**(): _Promise_<[_RoutingChannel_](../modules.md#routingchannel)\>

**Returns:** _Promise_<[_RoutingChannel_](../modules.md#routingchannel)\>

Defined in: [core-ethereum/src/index.ts:77](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L77)

---

### smartContractInfo

▸ **smartContractInfo**(): _string_

**Returns:** _string_

Defined in: [core-ethereum/src/index.ts:112](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L112)

---

### stop

▸ **stop**(): _Promise_<void\>

Stops the connector.

**Returns:** _Promise_<void\>

Defined in: [core-ethereum/src/index.ts:44](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L44)

---

### uncachedGetBalance

▸ `Private` **uncachedGetBalance**(): _Promise_<Balance\>

**Returns:** _Promise_<Balance\>

Defined in: [core-ethereum/src/index.ts:81](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L81)

---

### uncachedGetNativeBalance

▸ `Private` **uncachedGetNativeBalance**(): _Promise_<NativeBalance\>

Retrieves ETH balance, optionally uses the cache.

**Returns:** _Promise_<NativeBalance\>

ETH balance

Defined in: [core-ethereum/src/index.ts:103](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L103)

---

### waitForPublicNodes

▸ **waitForPublicNodes**(): _Promise_<Multiaddr[]\>

**Returns:** _Promise_<Multiaddr[]\>

Defined in: [core-ethereum/src/index.ts:116](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L116)

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

Defined in: [core-ethereum/src/index.ts:57](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L57)

---

### create

▸ `Static` **create**(`db`: _HoprDB_, `privateKey`: _Uint8Array_, `options?`: { `maxConfirmations?`: _number_ ; `provider?`: _string_ }): _Promise_<[_default_](default.md)\>

Creates an uninitialised instance.

#### Parameters

| Name                        | Type         | Description                                            |
| :-------------------------- | :----------- | :----------------------------------------------------- |
| `db`                        | _HoprDB_     | database instance                                      |
| `privateKey`                | _Uint8Array_ | that is used to derive that on-chain identity          |
| `options?`                  | _object_     | -                                                      |
| `options.maxConfirmations?` | _number_     | -                                                      |
| `options.provider?`         | _string_     | provider URI that is used to connect to the blockchain |

**Returns:** _Promise_<[_default_](default.md)\>

a promise resolved to the connector

Defined in: [core-ethereum/src/index.ts:128](https://github.com/hoprnet/hoprnet/blob/master/packages/core-ethereum/src/index.ts#L128)
