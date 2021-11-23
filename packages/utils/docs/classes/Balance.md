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
- [SYMBOL](Balance.md#symbol)

### Methods

- [add](Balance.md#add)
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
- [ZERO](Balance.md#zero)
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

#### Defined in

[types/primitives.ts:274](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L274)

## Properties

### bn

• `Protected` **bn**: `BN`

#### Inherited from

BalanceBase.bn

___

### symbol

• `Readonly` **symbol**: `string` = `Balance.SYMBOL`

#### Overrides

BalanceBase.symbol

#### Defined in

[types/primitives.ts:319](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L319)

___

### DECIMALS

▪ `Static` `Readonly` **DECIMALS**: `number` = `18`

#### Inherited from

BalanceBase.DECIMALS

#### Defined in

[types/primitives.ts:271](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L271)

___

### SIZE

▪ `Static` `Readonly` **SIZE**: `number` = `32`

#### Inherited from

BalanceBase.SIZE

#### Defined in

[types/primitives.ts:270](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L270)

___

### SYMBOL

▪ `Static` **SYMBOL**: `string` = `'txHOPR'`

#### Defined in

[types/primitives.ts:318](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L318)

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

#### Defined in

[types/primitives.ts:321](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L321)

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

#### Defined in

[types/primitives.ts:291](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L291)

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

#### Defined in

[types/primitives.ts:295](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L295)

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

#### Defined in

[types/primitives.ts:287](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L287)

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

#### Defined in

[types/primitives.ts:299](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L299)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Inherited from

BalanceBase.serialize

#### Defined in

[types/primitives.ts:303](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L303)

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

#### Defined in

[types/primitives.ts:325](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L325)

___

### toBN

▸ **toBN**(): `BN`

#### Returns

`BN`

#### Inherited from

BalanceBase.toBN

#### Defined in

[types/primitives.ts:279](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L279)

___

### toFormattedString

▸ **toFormattedString**(): `string`

#### Returns

`string`

#### Inherited from

BalanceBase.toFormattedString

#### Defined in

[types/primitives.ts:311](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L311)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Inherited from

BalanceBase.toHex

#### Defined in

[types/primitives.ts:283](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L283)

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Inherited from

BalanceBase.toString

#### Defined in

[types/primitives.ts:307](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L307)

___

### ZERO

▸ `Static` **ZERO**(): [`Balance`](Balance.md)

#### Returns

[`Balance`](Balance.md)

#### Defined in

[types/primitives.ts:333](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L333)

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

[types/primitives.ts:329](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L329)
