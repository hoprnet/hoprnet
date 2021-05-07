[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / u8a/u8aAdd

# Module: u8a/u8aAdd

## Table of contents

### Functions

- [u8aAdd](u8a_u8aadd.md#u8aadd)

## Functions

### u8aAdd

▸ **u8aAdd**(`inplace`: _boolean_, `a`: Uint8Array, `b`: Uint8Array): Uint8Array

Adds the contents of two arrays together while ignoring the final overflow.
Computes `a + b % ( 2 ** (8 * a.length) - 1)`

**`example`**
u8aAdd(false, new Uint8Array([1], new Uint8Array([2])) // Uint8Array([3])
u8aAdd(false, new Uint8Array([1], new Uint8Array([255])) // Uint8Array([0])
u8aAdd(false, new Uint8Array([0, 1], new Uint8Array([0, 255])) // Uint8Array([1, 0])

#### Parameters

| Name      | Type       | Description                          |
| :-------- | :--------- | :----------------------------------- |
| `inplace` | _boolean_  | result is stored in a if set to true |
| `a`       | Uint8Array | first array                          |
| `b`       | Uint8Array | second array                         |

**Returns:** Uint8Array

Defined in: [u8a/u8aAdd.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/u8a/u8aAdd.ts#L13)
