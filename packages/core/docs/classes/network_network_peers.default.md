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

\+ **new default**(`existingPeers`: *PeerId*[], `exclude?`: *PeerId*[]): [*default*](network_network_peers.default.md)

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `existingPeers` | *PeerId*[] | - |
| `exclude` | *PeerId*[] | [] |

**Returns:** [*default*](network_network_peers.default.md)

Defined in: [packages/core/src/network/network-peers.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L24)

## Properties

### peers

• `Private` **peers**: Entry[]

Defined in: [packages/core/src/network/network-peers.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L20)

## Methods

### all

▸ **all**(): *PeerId*[]

**Returns:** *PeerId*[]

Defined in: [packages/core/src/network/network-peers.ts:100](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L100)

___

### debugLog

▸ **debugLog**(): *string*

**Returns:** *string*

Defined in: [packages/core/src/network/network-peers.ts:108](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L108)

___

### find

▸ `Private` **find**(`peer`: *PeerId*): Entry

#### Parameters

| Name | Type |
| :------ | :------ |
| `peer` | *PeerId* |

**Returns:** Entry

Defined in: [packages/core/src/network/network-peers.ts:22](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L22)

___

### has

▸ **has**(`peer`: *PeerId*): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `peer` | *PeerId* |

**Returns:** *boolean*

Defined in: [packages/core/src/network/network-peers.ts:104](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L104)

___

### length

▸ **length**(): *number*

**Returns:** *number*

Defined in: [packages/core/src/network/network-peers.ts:96](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L96)

___

### nextPing

▸ `Private` **nextPing**(`e`: Entry): *number*

#### Parameters

| Name | Type |
| :------ | :------ |
| `e` | Entry |

**Returns:** *number*

Defined in: [packages/core/src/network/network-peers.ts:34](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L34)

___

### ping

▸ **ping**(`peer`: *PeerId*, `interaction`: (`peerID`: *PeerId*) => *Promise*<boolean\>): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `peer` | *PeerId* |
| `interaction` | (`peerID`: *PeerId*) => *Promise*<boolean\> |

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/network/network-peers.ts:57](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L57)

___

### pingSince

▸ **pingSince**(`thresholdTime`: *number*): *PeerId*[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `thresholdTime` | *number* |

**Returns:** *PeerId*[]

Defined in: [packages/core/src/network/network-peers.ts:53](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L53)

___

### qualityOf

▸ **qualityOf**(`peer`: *PeerId*): *number*

#### Parameters

| Name | Type |
| :------ | :------ |
| `peer` | *PeerId* |

**Returns:** *number*

Defined in: [packages/core/src/network/network-peers.ts:42](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L42)

___

### randomSubset

▸ **randomSubset**(`size`: *number*, `filter?`: (`peer`: *PeerId*) => *boolean*): *PeerId*[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `size` | *number* |
| `filter?` | (`peer`: *PeerId*) => *boolean* |

**Returns:** *PeerId*[]

Defined in: [packages/core/src/network/network-peers.ts:75](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L75)

___

### register

▸ **register**(`id`: *PeerId*): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `id` | *PeerId* |

**Returns:** *void*

Defined in: [packages/core/src/network/network-peers.ts:83](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/network-peers.ts#L83)
