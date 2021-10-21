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

[types/primitives.ts:261](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L261)

## Accessors

### DECIMALS

• `Static` `get` **DECIMALS**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:267](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L267)

___

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:295](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L295)

___

### SYMBOL

• `Static` `get` **SYMBOL**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:263](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L263)

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

[types/primitives.ts:279](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L279)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/primitives.ts:287](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L287)

___

### toBN

▸ **toBN**(): `BN`

#### Returns

`BN`

#### Defined in

[types/primitives.ts:271](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L271)

___

### toFormattedString

▸ **toFormattedString**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:291](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L291)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:275](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L275)

___

### ZERO

▸ `Static` **ZERO**(): [`Balance`](Balance.md)

#### Returns

[`Balance`](Balance.md)

#### Defined in

[types/primitives.ts:300](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L300)

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

[types/primitives.ts:283](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L283)
