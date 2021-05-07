[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / crypto/randomInteger

# Module: crypto/randomInteger

## Table of contents

### Functions

- [randomChoice](crypto_randominteger.md#randomchoice)
- [randomInteger](crypto_randominteger.md#randominteger)

## Functions

### randomChoice

▸ **randomChoice**<T\>(`collection`: T[]): T

#### Type parameters

| Name |
| :------ |
| `T` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `collection` | T[] |

**Returns:** T

Defined in: [crypto/randomInteger.ts:85](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/randomInteger.ts#L85)

___

### randomInteger

▸ **randomInteger**(`start`: *number*, `end?`: *number*, `_seed?`: Uint8Array): *number*

Returns a random value between `start` and `end`.

**`example`**
```
randomInteger(3) // result in { 0, 1, 2, 3 }
randomInteger(0, 3) // result in { 0, 1, 2, 3 }
randomInteger(7, 9) // result in { 7, 8, 9 }
randomInteger(8, 8) == 8
```

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `start` | *number* | start of the interval |
| `end?` | *number* | end of the interval inclusive |
| `_seed?` | Uint8Array | - |

**Returns:** *number*

random number between @param start and @param end

Defined in: [crypto/randomInteger.ts:18](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/randomInteger.ts#L18)
