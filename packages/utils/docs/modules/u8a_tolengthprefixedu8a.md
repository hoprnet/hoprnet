[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / u8a/toLengthPrefixedU8a

# Module: u8a/toLengthPrefixedU8a

## Table of contents

### Functions

- [toLengthPrefixedU8a](u8a_tolengthprefixedu8a.md#tolengthprefixedu8a)

## Functions

### toLengthPrefixedU8a

▸ **toLengthPrefixedU8a**(`arg`: Uint8Array, `additionalPadding?`: Uint8Array, `length?`: *number*): *Uint8Array*

Adds a length-prefix to a Uint8Array

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `arg` | Uint8Array | data to add padding |
| `additionalPadding?` | Uint8Array | optional additional padding that is inserted between length and data |
| `length?` | *number* | optional target length |

**Returns:** *Uint8Array*

Defined in: [u8a/toLengthPrefixedU8a.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/u8a/toLengthPrefixedU8a.ts#L12)
