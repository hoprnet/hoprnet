[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / NativeBalance

# Class: NativeBalance

## Table of contents

### Constructors

- [constructor](NativeBalance.md#constructor)

### Accessors

- [DECIMALS](NativeBalance.md#decimals)
- [SIZE](NativeBalance.md#size)
- [SYMBOL](NativeBalance.md#symbol)

### Methods

- [serialize](NativeBalance.md#serialize)
- [toBN](NativeBalance.md#tobn)
- [toFormattedString](NativeBalance.md#toformattedstring)
- [toHex](NativeBalance.md#tohex)
- [deserialize](NativeBalance.md#deserialize)

## Constructors

### constructor

• **new NativeBalance**(`bn`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `bn` | `BN` |

#### Defined in

[types/primitives.ts:322](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L322)

## Accessors

### DECIMALS

• `Static` `get` **DECIMALS**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:328](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L328)

___

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:352](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L352)

___

### SYMBOL

• `Static` `get` **SYMBOL**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:324](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L324)

## Methods

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/primitives.ts:344](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L344)

___

### toBN

▸ **toBN**(): `BN`

#### Returns

`BN`

#### Defined in

[types/primitives.ts:340](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L340)

___

### toFormattedString

▸ **toFormattedString**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:348](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L348)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:332](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L332)

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

[types/primitives.ts:336](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L336)
