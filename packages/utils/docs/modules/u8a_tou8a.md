[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / u8a/toU8a

# Module: u8a/toU8a

## Table of contents

### Functions

- [stringToU8a](u8a_tou8a.md#stringtou8a)
- [toU8a](u8a_tou8a.md#tou8a)

## Functions

### stringToU8a

▸ **stringToU8a**(`str`: _string_, `length?`: _number_): Uint8Array

Converts a **HEX** string to a Uint8Array and optionally adds some padding to match
the desired size.

**`example`**
stringToU8a('0xDEadBeeF') // Uint8Array [ 222, 173, 190, 239 ]

**`notice`** Throws an error in case a length was provided and the result does not fit.

#### Parameters

| Name      | Type     | Description                      |
| :-------- | :------- | :------------------------------- |
| `str`     | _string_ | string to convert                |
| `length?` | _number_ | desired length of the Uint8Array |

**Returns:** Uint8Array

Defined in: [u8a/toU8a.ts:60](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/u8a/toU8a.ts#L60)

---

### toU8a

▸ **toU8a**(`arg`: _number_, `length?`: _number_): Uint8Array

Converts a number to a Uint8Array and optionally adds some padding to match
the desired size.

#### Parameters

| Name      | Type     | Description                      |
| :-------- | :------- | :------------------------------- |
| `arg`     | _number_ | to convert to Uint8Array         |
| `length?` | _number_ | desired length of the Uint8Array |

**Returns:** Uint8Array

Defined in: [u8a/toU8a.ts:7](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/u8a/toU8a.ts#L7)
