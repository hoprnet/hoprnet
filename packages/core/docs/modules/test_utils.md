[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / test-utils

# Module: test-utils

## Table of contents

### Type aliases

- [LibP2PMocks](test_utils.md#libp2pmocks)

### Functions

- [connectionHelper](test_utils.md#connectionhelper)
- [fakeAddress](test_utils.md#fakeaddress)
- [fakePeerId](test_utils.md#fakepeerid)
- [generateLibP2PMock](test_utils.md#generatelibp2pmock)
- [getAddress](test_utils.md#getaddress)

## Type aliases

### LibP2PMocks

Ƭ **LibP2PMocks**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `address` | Multiaddr |
| `node` | [*LibP2P*](../classes/index.libp2p-1.md) |

Defined in: [packages/core/src/test-utils.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/test-utils.ts#L29)

## Functions

### connectionHelper

▸ **connectionHelper**(`nodes`: [*LibP2P*](../classes/index.libp2p-1.md)[]): *void*

Informs each node about the others existence.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `nodes` | [*LibP2P*](../classes/index.libp2p-1.md)[] | Hopr nodes |

**Returns:** *void*

Defined in: [packages/core/src/test-utils.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/test-utils.ts#L12)

___

### fakeAddress

▸ **fakeAddress**(`id`: PeerId): Multiaddr

#### Parameters

| Name | Type |
| :------ | :------ |
| `id` | PeerId |

**Returns:** Multiaddr

Defined in: [packages/core/src/test-utils.ts:61](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/test-utils.ts#L61)

___

### fakePeerId

▸ **fakePeerId**(`i`: *number* \| *string*): PeerId

#### Parameters

| Name | Type |
| :------ | :------ |
| `i` | *number* \| *string* |

**Returns:** PeerId

Defined in: [packages/core/src/test-utils.ts:53](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/test-utils.ts#L53)

___

### generateLibP2PMock

▸ **generateLibP2PMock**(`addr?`: *string*): *Promise*<[*LibP2PMocks*](test_utils.md#libp2pmocks)\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `addr` | *string* | '/ip4/0.0.0.0/tcp/0' |

**Returns:** *Promise*<[*LibP2PMocks*](test_utils.md#libp2pmocks)\>

Defined in: [packages/core/src/test-utils.ts:34](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/test-utils.ts#L34)

___

### getAddress

▸ **getAddress**(`node`: [*LibP2P*](../classes/index.libp2p-1.md)): Multiaddr

#### Parameters

| Name | Type |
| :------ | :------ |
| `node` | [*LibP2P*](../classes/index.libp2p-1.md) |

**Returns:** Multiaddr

Defined in: [packages/core/src/test-utils.ts:21](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/test-utils.ts#L21)
