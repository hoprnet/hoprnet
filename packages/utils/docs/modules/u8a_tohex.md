[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / u8a/toHex

# Module: u8a/toHex

## Table of contents

### Functions

- [u8aToHex](u8a_tohex.md#u8atohex)

## Functions

### u8aToHex

▸ **u8aToHex**(`arr?`: Uint8Array, `prefixed?`: _boolean_): _string_

Converts a Uint8Array to a hex string.

**`notice`** Mainly used for debugging.

#### Parameters

| Name       | Type       | Default value | Description                           |
| :--------- | :--------- | :------------ | :------------------------------------ |
| `arr?`     | Uint8Array | -             | Uint8Array                            |
| `prefixed` | _boolean_  | true          | if `true` add a `0x` in the beginning |

**Returns:** _string_

Defined in: [u8a/toHex.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/u8a/toHex.ts#L8)
