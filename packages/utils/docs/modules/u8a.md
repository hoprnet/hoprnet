[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / u8a

# Module: u8a

## Table of contents

### References

- [A_EQUALS_B](u8a.md#a_equals_b)
- [A_STRICLY_LESS_THAN_B](u8a.md#a_stricly_less_than_b)
- [A_STRICTLY_GREATER_THAN_B](u8a.md#a_strictly_greater_than_b)
- [LENGTH_PREFIX_LENGTH](u8a.md#length_prefix_length)
- [lengthPrefixedToU8a](u8a.md#lengthprefixedtou8a)
- [stringToU8a](u8a.md#stringtou8a)
- [toLengthPrefixedU8a](u8a.md#tolengthprefixedu8a)
- [toU8a](u8a.md#tou8a)
- [u8aAdd](u8a.md#u8aadd)
- [u8aAllocate](u8a.md#u8aallocate)
- [u8aCompare](u8a.md#u8acompare)
- [u8aConcat](u8a.md#u8aconcat)
- [u8aEquals](u8a.md#u8aequals)
- [u8aToHex](u8a.md#u8atohex)
- [u8aToNumber](u8a.md#u8atonumber)
- [u8aXOR](u8a.md#u8axor)

### Type aliases

- [U8aAndSize](u8a.md#u8aandsize)

### Functions

- [serializeToU8a](u8a.md#serializetou8a)
- [u8aSplit](u8a.md#u8asplit)

## References

### A_EQUALS_B

Re-exports: [A_EQUALS_B](u8a_u8acompare.md#a_equals_b)

---

### A_STRICLY_LESS_THAN_B

Re-exports: [A_STRICLY_LESS_THAN_B](u8a_u8acompare.md#a_stricly_less_than_b)

---

### A_STRICTLY_GREATER_THAN_B

Re-exports: [A_STRICTLY_GREATER_THAN_B](u8a_u8acompare.md#a_strictly_greater_than_b)

---

### LENGTH_PREFIX_LENGTH

Re-exports: [LENGTH_PREFIX_LENGTH](u8a_constants.md#length_prefix_length)

---

### lengthPrefixedToU8a

Re-exports: [lengthPrefixedToU8a](u8a_lengthprefixedtou8a.md#lengthprefixedtou8a)

---

### stringToU8a

Re-exports: [stringToU8a](u8a_tou8a.md#stringtou8a)

---

### toLengthPrefixedU8a

Re-exports: [toLengthPrefixedU8a](u8a_tolengthprefixedu8a.md#tolengthprefixedu8a)

---

### toU8a

Re-exports: [toU8a](u8a_tou8a.md#tou8a)

---

### u8aAdd

Re-exports: [u8aAdd](u8a_u8aadd.md#u8aadd)

---

### u8aAllocate

Re-exports: [u8aAllocate](u8a_allocate.md#u8aallocate)

---

### u8aCompare

Re-exports: [u8aCompare](u8a_u8acompare.md#u8acompare)

---

### u8aConcat

Re-exports: [u8aConcat](u8a_concat.md#u8aconcat)

---

### u8aEquals

Re-exports: [u8aEquals](u8a_equals.md#u8aequals)

---

### u8aToHex

Re-exports: [u8aToHex](u8a_tohex.md#u8atohex)

---

### u8aToNumber

Re-exports: [u8aToNumber](u8a_u8atonumber.md#u8atonumber)

---

### u8aXOR

Re-exports: [u8aXOR](u8a_xor.md#u8axor)

## Type aliases

### U8aAndSize

Ƭ **U8aAndSize**: [Uint8Array, *number*]

Defined in: [u8a/index.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/u8a/index.ts#L20)

## Functions

### serializeToU8a

▸ **serializeToU8a**(`items`: [_U8aAndSize_](u8a.md#u8aandsize)[]): Uint8Array

#### Parameters

| Name    | Type                                |
| :------ | :---------------------------------- |
| `items` | [_U8aAndSize_](u8a.md#u8aandsize)[] |

**Returns:** Uint8Array

Defined in: [u8a/index.ts:22](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/u8a/index.ts#L22)

---

### u8aSplit

▸ **u8aSplit**(`u8a`: Uint8Array, `sizes`: _number_[]): Uint8Array[]

#### Parameters

| Name    | Type       |
| :------ | :--------- |
| `u8a`   | Uint8Array |
| `sizes` | _number_[] |

**Returns:** Uint8Array[]

Defined in: [u8a/index.ts:36](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/u8a/index.ts#L36)
