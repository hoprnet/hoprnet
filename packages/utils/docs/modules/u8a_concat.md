[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / u8a/concat

# Module: u8a/concat

## Table of contents

### Functions

- [u8aConcat](u8a_concat.md#u8aconcat)

## Functions

### u8aConcat

▸ **u8aConcat**(...`list`: (Uint8Array \| *undefined*)[]): Uint8Array

Concatenates the input arrays into a single `UInt8Array`.

**`example`**
u8aConcat(
  new Uint8Array([1, 1, 1]),
  new Uint8Array([2, 2, 2])
); // Uint8Arrau([1, 1, 1, 2, 2, 2])
 * u8aConcat(
  new Uint8Array([1, 1, 1]),
  undefined
  new Uint8Array([2, 2, 2])
); // Uint8Arrau([1, 1, 1, 2, 2, 2])

#### Parameters

| Name | Type |
| :------ | :------ |
| `...list` | (Uint8Array \| *undefined*)[] |

**Returns:** Uint8Array

Defined in: [u8a/concat.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/u8a/concat.ts#L15)
