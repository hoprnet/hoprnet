[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / SimpleCounter

# Class: SimpleCounter

Represents a simple monotonic unsigned integer counter.
Wrapper for IntCounter type

## Table of contents

### Constructors

- [constructor](SimpleCounter.md#constructor)

### Methods

- [free](SimpleCounter.md#free)
- [get](SimpleCounter.md#get)
- [increment](SimpleCounter.md#increment)
- [increment\_by](SimpleCounter.md#increment_by)
- [name](SimpleCounter.md#name)

## Constructors

### constructor

• **new SimpleCounter**()

## Methods

### free

▸ **free**(): `void`

#### Returns

`void`

#### Defined in

lib/utils_metrics.d.ts:238

___

### get

▸ **get**(): `bigint`

Retrieves the value of the counter

#### Returns

`bigint`

#### Defined in

lib/utils_metrics.d.ts:243

___

### increment

▸ **increment**(): `void`

Increments the counter by 1

#### Returns

`void`

#### Defined in

lib/utils_metrics.d.ts:252

___

### increment\_by

▸ **increment_by**(`by`): `void`

Increments the counter by the given number.

#### Parameters

| Name | Type |
| :------ | :------ |
| `by` | `bigint` |

#### Returns

`void`

#### Defined in

lib/utils_metrics.d.ts:248

___

### name

▸ **name**(): `string`

Returns the name of the counter given at construction.

#### Returns

`string`

#### Defined in

lib/utils_metrics.d.ts:257
