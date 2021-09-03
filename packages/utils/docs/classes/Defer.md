[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Defer

# Class: Defer<T\>

## Type parameters

| Name |
| :------ |
| `T` |

## Table of contents

### Constructors

- [constructor](Defer.md#constructor)

### Properties

- [\_reject](Defer.md#_reject)
- [\_resolve](Defer.md#_resolve)
- [promise](Defer.md#promise)

### Methods

- [reject](Defer.md#reject)
- [resolve](Defer.md#resolve)

## Constructors

### constructor

• **new Defer**<`T`\>()

#### Type parameters

| Name |
| :------ |
| `T` |

#### Defined in

[defer.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/defer.ts#L6)

## Properties

### \_reject

• `Private` **\_reject**: (`reason?`: `any`) => `void`

#### Type declaration

▸ (`reason?`): `void`

##### Parameters

| Name | Type |
| :------ | :------ |
| `reason?` | `any` |

##### Returns

`void`

#### Defined in

[defer.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/defer.ts#L4)

___

### \_resolve

• `Private` **\_resolve**: (`value?`: `T` \| `PromiseLike`<`T`\>) => `void`

#### Type declaration

▸ (`value?`): `void`

##### Parameters

| Name | Type |
| :------ | :------ |
| `value?` | `T` \| `PromiseLike`<`T`\> |

##### Returns

`void`

#### Defined in

[defer.ts:3](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/defer.ts#L3)

___

### promise

• **promise**: `Promise`<`T`\>

#### Defined in

[defer.ts:2](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/defer.ts#L2)

## Methods

### reject

▸ **reject**(`reason?`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `reason?` | `any` |

#### Returns

`void`

#### Defined in

[defer.ts:17](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/defer.ts#L17)

___

### resolve

▸ **resolve**(`value?`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `value?` | `T` \| `PromiseLike`<`T`\> |

#### Returns

`void`

#### Defined in

[defer.ts:13](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/defer.ts#L13)
