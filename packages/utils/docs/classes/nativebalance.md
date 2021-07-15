[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / NativeBalance

# Class: NativeBalance

## Table of contents

### Constructors

- [constructor](nativebalance.md#constructor)

### Accessors

- [DECIMALS](nativebalance.md#decimals)
- [SIZE](nativebalance.md#size)
- [SYMBOL](nativebalance.md#symbol)

### Methods

- [serialize](nativebalance.md#serialize)
- [toBN](nativebalance.md#tobn)
- [toFormattedString](nativebalance.md#toformattedstring)
- [toHex](nativebalance.md#tohex)
- [deserialize](nativebalance.md#deserialize)

## Constructors

### constructor

• **new NativeBalance**(`bn`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `bn` | `BN` |

#### Defined in

[types/primitives.ts:295](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L295)

## Accessors

### DECIMALS

• `Static` `get` **DECIMALS**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:302](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L302)

___

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:326](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L326)

___

### SYMBOL

• `Static` `get` **SYMBOL**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:298](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L298)

## Methods

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/primitives.ts:318](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L318)

___

### toBN

▸ **toBN**(): `BN`

#### Returns

`BN`

#### Defined in

[types/primitives.ts:314](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L314)

___

### toFormattedString

▸ **toFormattedString**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:322](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L322)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:306](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L306)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`NativeBalance`](nativebalance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`NativeBalance`](nativebalance.md)

#### Defined in

[types/primitives.ts:310](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L310)
