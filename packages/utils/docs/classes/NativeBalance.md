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
- [SYMBOL](NativeBalance.md#symbol-1)

### Accessors

- [ZERO](NativeBalance.md#zero)

### Methods

- [add](NativeBalance.md#add)
- [eq](NativeBalance.md#eq)
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

[src/types/primitives.ts:171](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L171)

## Properties

### bn

• `Protected` **bn**: `BN`

#### Inherited from

BalanceBase.bn

#### Defined in

[src/types/primitives.ts:171](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L171)

___

### symbol

• `Readonly` **symbol**: `string` = `NativeBalance.SYMBOL`

#### Overrides

BalanceBase.symbol

#### Defined in

[src/types/primitives.ts:241](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L241)

___

### DECIMALS

▪ `Static` `Readonly` **DECIMALS**: `number` = `18`

#### Inherited from

BalanceBase.DECIMALS

#### Defined in

[src/types/primitives.ts:168](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L168)

___

### SIZE

▪ `Static` `Readonly` **SIZE**: `number` = `32`

#### Inherited from

BalanceBase.SIZE

#### Defined in

[src/types/primitives.ts:167](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L167)

___

### SYMBOL

▪ `Static` **SYMBOL**: `string` = `'xDAI'`

#### Defined in

[src/types/primitives.ts:240](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L240)

## Accessors

### ZERO

• `Static` `get` **ZERO**(): [`NativeBalance`](NativeBalance.md)

#### Returns

[`NativeBalance`](NativeBalance.md)

#### Defined in

[src/types/primitives.ts:254](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L254)

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

[src/types/primitives.ts:243](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L243)

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

[src/types/primitives.ts:176](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L176)

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

[src/types/primitives.ts:192](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L192)

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

[src/types/primitives.ts:196](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L196)

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

[src/types/primitives.ts:188](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L188)

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

[src/types/primitives.ts:200](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L200)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Inherited from

BalanceBase.serialize

#### Defined in

[src/types/primitives.ts:204](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L204)

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

[src/types/primitives.ts:247](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L247)

___

### toBN

▸ **toBN**(): `BN`

#### Returns

`BN`

#### Inherited from

BalanceBase.toBN

#### Defined in

[src/types/primitives.ts:180](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L180)

___

### toFormattedString

▸ **toFormattedString**(): `string`

#### Returns

`string`

#### Inherited from

BalanceBase.toFormattedString

#### Defined in

[src/types/primitives.ts:212](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L212)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Inherited from

BalanceBase.toHex

#### Defined in

[src/types/primitives.ts:184](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L184)

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Inherited from

BalanceBase.toString

#### Defined in

[src/types/primitives.ts:208](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L208)

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

[src/types/primitives.ts:251](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L251)
