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

Ƭ **LibP2PMocks**: _object_

#### Type declaration

| Name      | Type                                     |
| :-------- | :--------------------------------------- |
| `address` | Multiaddr                                |
| `node`    | [_LibP2P_](../classes/index.libp2p-1.md) |

Defined in: [packages/core/src/test-utils.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/test-utils.ts#L29)

## Functions

### connectionHelper

▸ **connectionHelper**(`nodes`: [_LibP2P_](../classes/index.libp2p-1.md)[]): _void_

Informs each node about the others existence.

#### Parameters

| Name    | Type                                       | Description |
| :------ | :----------------------------------------- | :---------- |
| `nodes` | [_LibP2P_](../classes/index.libp2p-1.md)[] | Hopr nodes  |

**Returns:** _void_

Defined in: [packages/core/src/test-utils.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/test-utils.ts#L12)

---

### fakeAddress

▸ **fakeAddress**(`id`: PeerId): Multiaddr

#### Parameters

| Name | Type   |
| :--- | :----- |
| `id` | PeerId |

**Returns:** Multiaddr

Defined in: [packages/core/src/test-utils.ts:61](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/test-utils.ts#L61)

---

### fakePeerId

▸ **fakePeerId**(`i`: _number_ \| _string_): PeerId

#### Parameters

| Name | Type                 |
| :--- | :------------------- |
| `i`  | _number_ \| _string_ |

**Returns:** PeerId

Defined in: [packages/core/src/test-utils.ts:53](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/test-utils.ts#L53)

---

### generateLibP2PMock

▸ **generateLibP2PMock**(`addr?`: _string_): _Promise_<[_LibP2PMocks_](test_utils.md#libp2pmocks)\>

#### Parameters

| Name   | Type     | Default value        |
| :----- | :------- | :------------------- |
| `addr` | _string_ | '/ip4/0.0.0.0/tcp/0' |

**Returns:** _Promise_<[_LibP2PMocks_](test_utils.md#libp2pmocks)\>

Defined in: [packages/core/src/test-utils.ts:34](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/test-utils.ts#L34)

---

### getAddress

▸ **getAddress**(`node`: [_LibP2P_](../classes/index.libp2p-1.md)): Multiaddr

#### Parameters

| Name   | Type                                     |
| :----- | :--------------------------------------- |
| `node` | [_LibP2P_](../classes/index.libp2p-1.md) |

**Returns:** Multiaddr

Defined in: [packages/core/src/test-utils.ts:21](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/test-utils.ts#L21)
