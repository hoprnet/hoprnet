[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / crypto/sampleGroupElement

# Module: crypto/sampleGroupElement

## Table of contents

### Functions

- [sampleGroupElement](crypto_samplegroupelement.md#samplegroupelement)

## Functions

### sampleGroupElement

▸ **sampleGroupElement**(`compressed?`: _boolean_): [exponent: Uint8Array, groupElement: Uint8Array]

Samples a valid exponent and returns the exponent
and the product of exponent and base-point.

**`dev`** can be used to derive a secp256k1 keypair

#### Parameters

| Name         | Type      | Default value |
| :----------- | :-------- | :------------ |
| `compressed` | _boolean_ | false         |

**Returns:** [exponent: Uint8Array, groupElement: Uint8Array]

Defined in: [crypto/sampleGroupElement.ts:11](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/sampleGroupElement.ts#L11)
