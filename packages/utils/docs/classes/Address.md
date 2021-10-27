[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Address

# Class: Address

## Table of contents

### Constructors

- [constructor](Address.md#constructor)

### Accessors

- [SIZE](Address.md#size)

### Methods

- [compare](Address.md#compare)
- [eq](Address.md#eq)
- [lt](Address.md#lt)
- [serialize](Address.md#serialize)
- [sortPair](Address.md#sortpair)
- [toHex](Address.md#tohex)
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

[types/primitives.ts:102](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L102)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/primitives.ts:110](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L110)

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

[types/primitives.ts:134](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L134)

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

[types/primitives.ts:130](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L130)

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

[types/primitives.ts:138](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L138)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/primitives.ts:122](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L122)

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

[types/primitives.ts:142](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L142)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:126](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L126)

___

### createMock

▸ `Static` **createMock**(): [`Address`](Address.md)

#### Returns

[`Address`](Address.md)

#### Defined in

[types/primitives.ts:146](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L146)

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

[types/primitives.ts:118](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L118)

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

[types/primitives.ts:114](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L114)
