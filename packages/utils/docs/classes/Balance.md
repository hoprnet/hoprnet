[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Balance

# Class: Balance

## Hierarchy

- `BalanceBase`

  ↳ **`Balance`**

## Table of contents

### Constructors

- [constructor](Balance.md#constructor)

### Properties

- [bn](Balance.md#bn)
- [symbol](Balance.md#symbol)
- [DECIMALS](Balance.md#decimals)
- [SIZE](Balance.md#size)
- [SYMBOL](Balance.md#symbol-1)

### Accessors

- [ZERO](Balance.md#zero)

### Methods

- [add](Balance.md#add)
- [eq](Balance.md#eq)
- [gt](Balance.md#gt)
- [gte](Balance.md#gte)
- [lt](Balance.md#lt)
- [lte](Balance.md#lte)
- [serialize](Balance.md#serialize)
- [sub](Balance.md#sub)
- [toBN](Balance.md#tobn)
- [toFormattedString](Balance.md#toformattedstring)
- [toHex](Balance.md#tohex)
- [toString](Balance.md#tostring)
- [deserialize](Balance.md#deserialize)

## Constructors

### constructor

• **new Balance**(`bn`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `bn` | `BN` |

#### Inherited from

BalanceBase.constructor

## Properties

### bn

• `Protected` **bn**: `BN`

#### Inherited from

BalanceBase.bn

#### Defined in

[types/primitives.ts:171](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L171)

___

### symbol

• `Readonly` **symbol**: `string` = `Balance.SYMBOL`

#### Overrides

BalanceBase.symbol

#### Defined in

[types/primitives.ts:220](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L220)

___

### DECIMALS

▪ `Static` `Readonly` **DECIMALS**: `number` = `18`

#### Inherited from

BalanceBase.DECIMALS

#### Defined in

[types/primitives.ts:168](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L168)

___

### SIZE

▪ `Static` `Readonly` **SIZE**: `number` = `32`

#### Inherited from

BalanceBase.SIZE

#### Defined in

[types/primitives.ts:167](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L167)

___

### SYMBOL

▪ `Static` **SYMBOL**: `string` = `'txHOPR'`

#### Defined in

[types/primitives.ts:219](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L219)

## Accessors

### ZERO

• `Static` `get` **ZERO**(): [`Balance`](Balance.md)

#### Returns

[`Balance`](Balance.md)

## Methods

### add

▸ **add**(`b`): [`Balance`](Balance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Balance`](Balance.md) |

#### Returns

[`Balance`](Balance.md)

#### Overrides

BalanceBase.add

___

### eq

▸ **eq**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Balance`](Balance.md) |

#### Returns

`boolean`

#### Inherited from

BalanceBase.eq

___

### gt

▸ **gt**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | `BalanceBase` |

#### Returns

`boolean`

#### Inherited from

BalanceBase.gt

___

### gte

▸ **gte**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | `BalanceBase` |

#### Returns

`boolean`

#### Inherited from

BalanceBase.gte

___

### lt

▸ **lt**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | `BalanceBase` |

#### Returns

`boolean`

#### Inherited from

BalanceBase.lt

___

### lte

▸ **lte**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | `BalanceBase` |

#### Returns

`boolean`

#### Inherited from

BalanceBase.lte

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Inherited from

BalanceBase.serialize

___

### sub

▸ **sub**(`b`): [`Balance`](Balance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Balance`](Balance.md) |

#### Returns

[`Balance`](Balance.md)

#### Overrides

BalanceBase.sub

___

### toBN

▸ **toBN**(): `BN`

#### Returns

`BN`

#### Inherited from

BalanceBase.toBN

___

### toFormattedString

▸ **toFormattedString**(): `string`

#### Returns

`string`

#### Inherited from

BalanceBase.toFormattedString

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Inherited from

BalanceBase.toHex

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Inherited from

BalanceBase.toString

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`Balance`](Balance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`Balance`](Balance.md)
