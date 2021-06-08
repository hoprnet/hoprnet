[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Balance

# Class: Balance

## Table of contents

### Constructors

- [constructor](balance.md#constructor)

### Accessors

- [DECIMALS](balance.md#decimals)
- [SIZE](balance.md#size)
- [SYMBOL](balance.md#symbol)

### Methods

- [add](balance.md#add)
- [serialize](balance.md#serialize)
- [toBN](balance.md#tobn)
- [toFormattedString](balance.md#toformattedstring)
- [toHex](balance.md#tohex)
- [ZERO](balance.md#zero)
- [deserialize](balance.md#deserialize)

## Constructors

### constructor

• **new Balance**(`bn`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `bn` | `BN` |

#### Defined in

[types/primitives.ts:235](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L235)

## Accessors

### DECIMALS

• `Static` `get` **DECIMALS**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:242](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L242)

___

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:270](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L270)

___

### SYMBOL

• `Static` `get` **SYMBOL**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:238](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L238)

## Methods

### add

▸ **add**(`b`): [Balance](balance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [Balance](balance.md) |

#### Returns

[Balance](balance.md)

#### Defined in

[types/primitives.ts:254](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L254)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/primitives.ts:262](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L262)

___

### toBN

▸ **toBN**(): `BN`

#### Returns

`BN`

#### Defined in

[types/primitives.ts:246](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L246)

___

### toFormattedString

▸ **toFormattedString**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:266](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L266)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:250](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L250)

___

### ZERO

▸ `Static` **ZERO**(): [Balance](balance.md)

#### Returns

[Balance](balance.md)

#### Defined in

[types/primitives.ts:275](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L275)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [Balance](balance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[Balance](balance.md)

#### Defined in

[types/primitives.ts:258](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L258)
