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

### Accessors

- [ZERO](Balance.md#zero)

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

[types/primitives.ts:273](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L273)

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

[types/primitives.ts:318](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L318)

___

### DECIMALS

▪ `Static` `Readonly` **DECIMALS**: `number` = `18`

#### Inherited from

BalanceBase.DECIMALS

#### Defined in

[types/primitives.ts:270](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L270)

___

### SIZE

▪ `Static` `Readonly` **SIZE**: `number` = `32`

#### Inherited from

BalanceBase.SIZE

#### Defined in

[types/primitives.ts:269](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L269)

___

### SYMBOL

▪ `Static` **SYMBOL**: `string` = `'txHOPR'`

#### Defined in

[types/primitives.ts:317](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L317)

## Accessors

### ZERO

• `Static` `get` **ZERO**(): [`Balance`](Balance.md)

#### Returns

[`Balance`](Balance.md)

#### Defined in

[types/primitives.ts:332](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L332)

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

[types/primitives.ts:320](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L320)

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

[types/primitives.ts:290](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L290)

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

[types/primitives.ts:294](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L294)

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

[types/primitives.ts:286](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L286)

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

[types/primitives.ts:298](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L298)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Inherited from

BalanceBase.serialize

#### Defined in

[types/primitives.ts:302](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L302)

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

[types/primitives.ts:324](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L324)

___

### toBN

▸ **toBN**(): `BN`

#### Returns

`BN`

#### Inherited from

BalanceBase.toBN

#### Defined in

[types/primitives.ts:278](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L278)

___

### toFormattedString

▸ **toFormattedString**(): `string`

#### Returns

`string`

#### Inherited from

BalanceBase.toFormattedString

#### Defined in

[types/primitives.ts:310](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L310)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Inherited from

BalanceBase.toHex

#### Defined in

[types/primitives.ts:282](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L282)

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Inherited from

BalanceBase.toString

#### Defined in

[types/primitives.ts:306](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L306)

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

[types/primitives.ts:328](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L328)
