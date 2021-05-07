[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / u8a/lengthPrefixedToU8a

# Module: u8a/lengthPrefixedToU8a

## Table of contents

### Functions

- [lengthPrefixedToU8a](u8a_lengthprefixedtou8a.md#lengthprefixedtou8a)

## Functions

### lengthPrefixedToU8a

▸ **lengthPrefixedToU8a**(`arg`: Uint8Array, `additionalPadding?`: Uint8Array, `targetLength?`: *number*): *Uint8Array*

Decodes a length-prefixed array and returns the encoded data.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `arg` | Uint8Array | array to decode |
| `additionalPadding?` | Uint8Array | additional padding to remove |
| `targetLength?` | *number* | optional target length |

**Returns:** *Uint8Array*

Defined in: [u8a/lengthPrefixedToU8a.ts:11](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/u8a/lengthPrefixedToU8a.ts#L11)
