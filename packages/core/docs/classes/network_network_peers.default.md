[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / [network/network-peers](../modules/network_network_peers.md) / default

# Class: default

[network/network-peers](../modules/network_network_peers.md).default

## Table of contents

### Constructors

- [constructor](network_network_peers.default.md#constructor)

### Properties

- [peers](network_network_peers.default.md#peers)

### Methods

- [all](network_network_peers.default.md#all)
- [debugLog](network_network_peers.default.md#debuglog)
- [find](network_network_peers.default.md#find)
- [has](network_network_peers.default.md#has)
- [length](network_network_peers.default.md#length)
- [nextPing](network_network_peers.default.md#nextping)
- [ping](network_network_peers.default.md#ping)
- [pingSince](network_network_peers.default.md#pingsince)
- [qualityOf](network_network_peers.default.md#qualityof)
- [randomSubset](network_network_peers.default.md#randomsubset)
- [register](network_network_peers.default.md#register)

## Constructors

### constructor

\+ **new default**(`existingPeers`: _PeerId_[], `exclude?`: _PeerId_[]): [_default_](network_network_peers.default.md)

#### Parameters

| Name            | Type       | Default value |
| :-------------- | :--------- | :------------ |
| `existingPeers` | _PeerId_[] | -             |
| `exclude`       | _PeerId_[] | []            |

**Returns:** [_default_](network_network_peers.default.md)

Defined in: [packages/core/src/network/network-peers.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L24)

## Properties

### peers

• `Private` **peers**: Entry[]

Defined in: [packages/core/src/network/network-peers.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L20)

## Methods

### all

▸ **all**(): _PeerId_[]

**Returns:** _PeerId_[]

Defined in: [packages/core/src/network/network-peers.ts:100](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L100)

---

### debugLog

▸ **debugLog**(): _string_

**Returns:** _string_

Defined in: [packages/core/src/network/network-peers.ts:108](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L108)

---

### find

▸ `Private` **find**(`peer`: _PeerId_): Entry

#### Parameters

| Name   | Type     |
| :----- | :------- |
| `peer` | _PeerId_ |

**Returns:** Entry

Defined in: [packages/core/src/network/network-peers.ts:22](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L22)

---

### has

▸ **has**(`peer`: _PeerId_): _boolean_

#### Parameters

| Name   | Type     |
| :----- | :------- |
| `peer` | _PeerId_ |

**Returns:** _boolean_

Defined in: [packages/core/src/network/network-peers.ts:104](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L104)

---

### length

▸ **length**(): _number_

**Returns:** _number_

Defined in: [packages/core/src/network/network-peers.ts:96](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L96)

---

### nextPing

▸ `Private` **nextPing**(`e`: Entry): _number_

#### Parameters

| Name | Type  |
| :--- | :---- |
| `e`  | Entry |

**Returns:** _number_

Defined in: [packages/core/src/network/network-peers.ts:34](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L34)

---

### ping

▸ **ping**(`peer`: _PeerId_, `interaction`: (`peerID`: _PeerId_) => _Promise_<boolean\>): _Promise_<void\>

#### Parameters

| Name          | Type                                        |
| :------------ | :------------------------------------------ |
| `peer`        | _PeerId_                                    |
| `interaction` | (`peerID`: _PeerId_) => _Promise_<boolean\> |

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/network/network-peers.ts:57](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L57)

---

### pingSince

▸ **pingSince**(`thresholdTime`: _number_): _PeerId_[]

#### Parameters

| Name            | Type     |
| :-------------- | :------- |
| `thresholdTime` | _number_ |

**Returns:** _PeerId_[]

Defined in: [packages/core/src/network/network-peers.ts:53](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L53)

---

### qualityOf

▸ **qualityOf**(`peer`: _PeerId_): _number_

#### Parameters

| Name   | Type     |
| :----- | :------- |
| `peer` | _PeerId_ |

**Returns:** _number_

Defined in: [packages/core/src/network/network-peers.ts:42](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L42)

---

### randomSubset

▸ **randomSubset**(`size`: _number_, `filter?`: (`peer`: _PeerId_) => _boolean_): _PeerId_[]

#### Parameters

| Name      | Type                            |
| :-------- | :------------------------------ |
| `size`    | _number_                        |
| `filter?` | (`peer`: _PeerId_) => _boolean_ |

**Returns:** _PeerId_[]

Defined in: [packages/core/src/network/network-peers.ts:75](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L75)

---

### register

▸ **register**(`id`: _PeerId_): _void_

#### Parameters

| Name | Type     |
| :--- | :------- |
| `id` | _PeerId_ |

**Returns:** _void_

Defined in: [packages/core/src/network/network-peers.ts:83](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L83)
