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

[types/primitives.ts:238](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L238)

## Accessors

### DECIMALS

• `Static` `get` **DECIMALS**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:245](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L245)

___

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:273](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L273)

___

### SYMBOL

• `Static` `get` **SYMBOL**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:241](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L241)

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

[types/primitives.ts:257](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L257)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/primitives.ts:265](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L265)

___

### toBN

▸ **toBN**(): `BN`

#### Returns

`BN`

#### Defined in

[types/primitives.ts:249](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L249)

___

### toFormattedString

▸ **toFormattedString**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:269](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L269)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:253](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L253)

___

### ZERO

▸ `Static` **ZERO**(): [Balance](balance.md)

#### Returns

[Balance](balance.md)

#### Defined in

[types/primitives.ts:278](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L278)

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

[types/primitives.ts:261](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L261)
