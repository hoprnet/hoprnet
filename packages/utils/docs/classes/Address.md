[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Address

# Class: Address

## Table of contents

### Constructors

- [constructor](Address.md#constructor)

### Properties

- [arr](Address.md#arr)

### Accessors

- [SIZE](Address.md#size)

### Methods

- [compare](Address.md#compare)
- [eq](Address.md#eq)
- [lt](Address.md#lt)
- [serialize](Address.md#serialize)
- [sortPair](Address.md#sortpair)
- [toBytes32](Address.md#tobytes32)
- [toHex](Address.md#tohex)
- [toString](Address.md#tostring)
- [createMock](Address.md#createmock)
- [deserialize](Address.md#deserialize)
- [fromString](Address.md#fromstring)

## Constructors

### constructor

• **new Address**(`arr`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Defined in

[src/types/primitives.ts:10](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L10)

## Properties

### arr

• `Private` **arr**: `Uint8Array`

#### Defined in

[src/types/primitives.ts:10](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L10)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[src/types/primitives.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L16)

## Methods

### compare

▸ **compare**(`b`): `number`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Address`](Address.md) |

#### Returns

`number`

#### Defined in

[src/types/primitives.ts:48](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L48)

___

### eq

▸ **eq**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Address`](Address.md) |

#### Returns

`boolean`

#### Defined in

[src/types/primitives.ts:44](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L44)

___

### lt

▸ **lt**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Address`](Address.md) |

#### Returns

`boolean`

#### Defined in

[src/types/primitives.ts:52](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L52)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[src/types/primitives.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L28)

___

### sortPair

▸ **sortPair**(`b`): [[`Address`](Address.md), [`Address`](Address.md)]

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`Address`](Address.md) |

#### Returns

[[`Address`](Address.md), [`Address`](Address.md)]

#### Defined in

[src/types/primitives.ts:56](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L56)

___

### toBytes32

▸ **toBytes32**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[src/types/primitives.ts:40](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L40)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[src/types/primitives.ts:32](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L32)

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Defined in

[src/types/primitives.ts:36](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L36)

___

### createMock

▸ `Static` **createMock**(): [`Address`](Address.md)

#### Returns

[`Address`](Address.md)

#### Defined in

[src/types/primitives.ts:60](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L60)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`Address`](Address.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`Address`](Address.md)

#### Defined in

[src/types/primitives.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L24)

___

### fromString

▸ `Static` **fromString**(`str`): [`Address`](Address.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[`Address`](Address.md)

#### Defined in

[src/types/primitives.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L20)
