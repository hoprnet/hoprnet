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

▸ **iterateHash**(`seed`: Uint8Array \| _undefined_, `hashFunc`: (`preImage`: Uint8Array) => _Promise_<Uint8Array\> \| Uint8Array, `iterations`: _number_, `stepSize`: _number_, `hint?`: (`index`: _number_) => Uint8Array \| _undefined_ \| _Promise_<Uint8Array \| undefined\>): _Promise_<{ `hash`: Uint8Array ; `intermediates`: [_Intermediate_](../interfaces/crypto_hashiterator.intermediate.md)[] }\>

#### Parameters

| Name         | Type                                                                                    |
| :----------- | :-------------------------------------------------------------------------------------- |
| `seed`       | Uint8Array \| _undefined_                                                               |
| `hashFunc`   | (`preImage`: Uint8Array) => _Promise_<Uint8Array\> \| Uint8Array                        |
| `iterations` | _number_                                                                                |
| `stepSize`   | _number_                                                                                |
| `hint?`      | (`index`: _number_) => Uint8Array \| _undefined_ \| _Promise_<Uint8Array \| undefined\> |

**Returns:** _Promise_<{ `hash`: Uint8Array ; `intermediates`: [_Intermediate_](../interfaces/crypto_hashiterator.intermediate.md)[] }\>

Defined in: [crypto/hashIterator.ts:7](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/hashIterator.ts#L7)

---

### recoverIteratedHash

▸ **recoverIteratedHash**(`hashValue`: Uint8Array, `hashFunc`: (`preImage`: Uint8Array) => _Promise_<Uint8Array\> \| Uint8Array, `hint`: (`index`: _number_) => _Promise_<Uint8Array\>, `maxIterations`: _number_, `stepSize?`: _number_, `indexHint?`: _number_): _Promise_<[_Intermediate_](../interfaces/crypto_hashiterator.intermediate.md) \| undefined\>

#### Parameters

| Name            | Type                                                             |
| :-------------- | :--------------------------------------------------------------- |
| `hashValue`     | Uint8Array                                                       |
| `hashFunc`      | (`preImage`: Uint8Array) => _Promise_<Uint8Array\> \| Uint8Array |
| `hint`          | (`index`: _number_) => _Promise_<Uint8Array\>                    |
| `maxIterations` | _number_                                                         |
| `stepSize?`     | _number_                                                         |
| `indexHint?`    | _number_                                                         |

**Returns:** _Promise_<[_Intermediate_](../interfaces/crypto_hashiterator.intermediate.md) \| undefined\>

Defined in: [crypto/hashIterator.ts:55](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/hashIterator.ts#L55)
