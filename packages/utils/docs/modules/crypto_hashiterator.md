[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / crypto/hashIterator

# Module: crypto/hashIterator

## Table of contents

### Interfaces

- [Intermediate](../interfaces/crypto_hashiterator.intermediate.md)

### Functions

- [iterateHash](crypto_hashiterator.md#iteratehash)
- [recoverIteratedHash](crypto_hashiterator.md#recoveriteratedhash)

## Functions

### iterateHash

▸ **iterateHash**(`seed`: Uint8Array \| *undefined*, `hashFunc`: (`preImage`: Uint8Array) => *Promise*<Uint8Array\> \| Uint8Array, `iterations`: *number*, `stepSize`: *number*, `hint?`: (`index`: *number*) => Uint8Array \| *undefined* \| *Promise*<Uint8Array \| undefined\>): *Promise*<{ `hash`: Uint8Array ; `intermediates`: [*Intermediate*](../interfaces/crypto_hashiterator.intermediate.md)[]  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `seed` | Uint8Array \| *undefined* |
| `hashFunc` | (`preImage`: Uint8Array) => *Promise*<Uint8Array\> \| Uint8Array |
| `iterations` | *number* |
| `stepSize` | *number* |
| `hint?` | (`index`: *number*) => Uint8Array \| *undefined* \| *Promise*<Uint8Array \| undefined\> |

**Returns:** *Promise*<{ `hash`: Uint8Array ; `intermediates`: [*Intermediate*](../interfaces/crypto_hashiterator.intermediate.md)[]  }\>

Defined in: [crypto/hashIterator.ts:7](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/hashIterator.ts#L7)

___

### recoverIteratedHash

▸ **recoverIteratedHash**(`hashValue`: Uint8Array, `hashFunc`: (`preImage`: Uint8Array) => *Promise*<Uint8Array\> \| Uint8Array, `hint`: (`index`: *number*) => *Promise*<Uint8Array\>, `maxIterations`: *number*, `stepSize?`: *number*, `indexHint?`: *number*): *Promise*<[*Intermediate*](../interfaces/crypto_hashiterator.intermediate.md) \| undefined\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `hashValue` | Uint8Array |
| `hashFunc` | (`preImage`: Uint8Array) => *Promise*<Uint8Array\> \| Uint8Array |
| `hint` | (`index`: *number*) => *Promise*<Uint8Array\> |
| `maxIterations` | *number* |
| `stepSize?` | *number* |
| `indexHint?` | *number* |

**Returns:** *Promise*<[*Intermediate*](../interfaces/crypto_hashiterator.intermediate.md) \| undefined\>

Defined in: [crypto/hashIterator.ts:55](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/hashIterator.ts#L55)
