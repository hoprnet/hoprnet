[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Defer

# Class: Defer<T\>

## Type parameters

| Name |
| :------ |
| `T` |

## Table of contents

### Constructors

- [constructor](defer.md#constructor)

### Properties

- [\_reject](defer.md#_reject)
- [\_resolve](defer.md#_resolve)
- [promise](defer.md#promise)

### Methods

- [reject](defer.md#reject)
- [resolve](defer.md#resolve)

## Constructors

### constructor

• **new Defer**<`T`\>()

#### Type parameters

| Name |
| :------ |
| `T` |

#### Defined in

[defer.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/defer.ts#L5)

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

[defer.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/defer.ts#L5)

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

[defer.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/defer.ts#L4)

___

### promise

• **promise**: `Promise`<`T`\>

#### Defined in

[defer.ts:3](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/defer.ts#L3)

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

[defer.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/defer.ts#L18)

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

[defer.ts:14](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/defer.ts#L14)
