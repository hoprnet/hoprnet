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

[types/primitives.ts:280](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L280)

## Accessors

### DECIMALS

• `Static` `get` **DECIMALS**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:287](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L287)

___

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:311](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L311)

___

### SYMBOL

• `Static` `get` **SYMBOL**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:283](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L283)

## Methods

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/primitives.ts:303](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L303)

___

### toBN

▸ **toBN**(): `BN`

#### Returns

`BN`

#### Defined in

[types/primitives.ts:299](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L299)

___

### toFormattedString

▸ **toFormattedString**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:307](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L307)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:291](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L291)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [NativeBalance](nativebalance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[NativeBalance](nativebalance.md)

#### Defined in

[types/primitives.ts:295](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L295)
