[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Balance

# Class: Balance

## Table of contents

### Constructors

- [constructor](Balance.md#constructor)

### Accessors

- [DECIMALS](Balance.md#decimals)
- [SIZE](Balance.md#size)
- [SYMBOL](Balance.md#symbol)

### Methods

- [add](Balance.md#add)
- [serialize](Balance.md#serialize)
- [toBN](Balance.md#tobn)
- [toFormattedString](Balance.md#toformattedstring)
- [toHex](Balance.md#tohex)
- [ZERO](Balance.md#zero)
- [deserialize](Balance.md#deserialize)

## Constructors

### constructor

• **new Balance**(`bn`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `bn` | `BN` |

#### Defined in

[types/primitives.ts:255](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L255)

## Accessors

### DECIMALS

• `Static` `get` **DECIMALS**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:261](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L261)

___

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:289](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L289)

___

### SYMBOL

• `Static` `get` **SYMBOL**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:257](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L257)

## Methods

### add

▸ **add**(`b`): [`Balance`](Balance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Balance`](Balance.md) |

#### Returns

[`Balance`](Balance.md)

#### Defined in

[types/primitives.ts:273](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L273)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/primitives.ts:281](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L281)

___

### toBN

▸ **toBN**(): `BN`

#### Returns

`BN`

#### Defined in

[types/primitives.ts:265](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L265)

___

### toFormattedString

▸ **toFormattedString**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:285](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L285)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:269](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L269)

___

### ZERO

▸ `Static` **ZERO**(): [`Balance`](Balance.md)

#### Returns

[`Balance`](Balance.md)

#### Defined in

[types/primitives.ts:294](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L294)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`Balance`](Balance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`Balance`](Balance.md)

#### Defined in

[types/primitives.ts:277](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L277)
