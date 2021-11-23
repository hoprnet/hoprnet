[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / NativeBalance

# Class: NativeBalance

## Hierarchy

- `BalanceBase`

  ↳ **`NativeBalance`**

## Table of contents

### Constructors

- [constructor](NativeBalance.md#constructor)

### Properties

- [bn](NativeBalance.md#bn)
- [symbol](NativeBalance.md#symbol)
- [DECIMALS](NativeBalance.md#decimals)
- [SIZE](NativeBalance.md#size)
- [SYMBOL](NativeBalance.md#symbol)

### Methods

- [add](NativeBalance.md#add)
- [gt](NativeBalance.md#gt)
- [gte](NativeBalance.md#gte)
- [lt](NativeBalance.md#lt)
- [lte](NativeBalance.md#lte)
- [serialize](NativeBalance.md#serialize)
- [sub](NativeBalance.md#sub)
- [toBN](NativeBalance.md#tobn)
- [toFormattedString](NativeBalance.md#toformattedstring)
- [toHex](NativeBalance.md#tohex)
- [toString](NativeBalance.md#tostring)
- [ZERO](NativeBalance.md#zero)
- [deserialize](NativeBalance.md#deserialize)

## Constructors

### constructor

• **new NativeBalance**(`bn`)

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

• `Readonly` **symbol**: `string` = `NativeBalance.SYMBOL`

#### Overrides

BalanceBase.symbol

#### Defined in

[types/primitives.ts:340](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L340)

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

▪ `Static` **SYMBOL**: `string` = `'xDAI'`

#### Defined in

[types/primitives.ts:339](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L339)

## Methods

### add

▸ **add**(`b`): [`NativeBalance`](NativeBalance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`NativeBalance`](NativeBalance.md) |

#### Returns

[`NativeBalance`](NativeBalance.md)

#### Overrides

BalanceBase.add

#### Defined in

[types/primitives.ts:342](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L342)

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

▸ **sub**(`b`): [`NativeBalance`](NativeBalance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`NativeBalance`](NativeBalance.md) |

#### Returns

[`NativeBalance`](NativeBalance.md)

#### Overrides

BalanceBase.sub

#### Defined in

[types/primitives.ts:346](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L346)

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

▸ `Static` **ZERO**(): [`NativeBalance`](NativeBalance.md)

#### Returns

[`NativeBalance`](NativeBalance.md)

#### Defined in

[types/primitives.ts:353](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L353)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`NativeBalance`](NativeBalance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`NativeBalance`](NativeBalance.md)

#### Defined in

[types/primitives.ts:350](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L350)
