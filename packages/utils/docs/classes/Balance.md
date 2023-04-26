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

#### Defined in

[utils/src/types/primitives.ts:171](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L171)

## Properties

### bn

• `Protected` **bn**: `BN`

#### Inherited from

BalanceBase.bn

#### Defined in

[utils/src/types/primitives.ts:171](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L171)

___

### symbol

• `Readonly` **symbol**: `string` = `Balance.SYMBOL`

#### Overrides

BalanceBase.symbol

#### Defined in

[utils/src/types/primitives.ts:220](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L220)

___

### DECIMALS

▪ `Static` `Readonly` **DECIMALS**: `number` = `18`

#### Inherited from

BalanceBase.DECIMALS

#### Defined in

[utils/src/types/primitives.ts:168](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L168)

___

### SIZE

▪ `Static` `Readonly` **SIZE**: `number` = `32`

#### Inherited from

BalanceBase.SIZE

#### Defined in

[utils/src/types/primitives.ts:167](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L167)

___

### SYMBOL

▪ `Static` **SYMBOL**: `string` = `'txHOPR'`

#### Defined in

[utils/src/types/primitives.ts:219](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L219)

## Accessors

### ZERO

• `Static` `get` **ZERO**(): [`Balance`](Balance.md)

#### Returns

[`Balance`](Balance.md)

#### Defined in

[utils/src/types/primitives.ts:234](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L234)

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

[utils/src/types/primitives.ts:222](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L222)

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

#### Defined in

[utils/src/types/primitives.ts:176](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L176)

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

[utils/src/types/primitives.ts:192](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L192)

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

[utils/src/types/primitives.ts:196](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L196)

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

[utils/src/types/primitives.ts:188](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L188)

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

[utils/src/types/primitives.ts:200](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L200)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Inherited from

BalanceBase.serialize

#### Defined in

[utils/src/types/primitives.ts:204](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L204)

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

[utils/src/types/primitives.ts:226](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L226)

___

### toBN

▸ **toBN**(): `BN`

#### Returns

`BN`

#### Inherited from

BalanceBase.toBN

#### Defined in

[utils/src/types/primitives.ts:180](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L180)

___

### toFormattedString

▸ **toFormattedString**(): `string`

#### Returns

`string`

#### Inherited from

BalanceBase.toFormattedString

#### Defined in

[utils/src/types/primitives.ts:212](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L212)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Inherited from

BalanceBase.toHex

#### Defined in

[utils/src/types/primitives.ts:184](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L184)

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Inherited from

BalanceBase.toString

#### Defined in

[utils/src/types/primitives.ts:208](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L208)

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

[utils/src/types/primitives.ts:230](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L230)
