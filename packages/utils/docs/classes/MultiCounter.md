[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / MultiCounter

# Class: MultiCounter

Represents a vector of named monotonic unsigned integer counters.
Wrapper for IntCounterVec type

## Table of contents

### Constructors

- [constructor](MultiCounter.md#constructor)

### Methods

- [free](MultiCounter.md#free)
- [get](MultiCounter.md#get)
- [increment](MultiCounter.md#increment)
- [increment\_by](MultiCounter.md#increment_by)
- [name](MultiCounter.md#name)

## Constructors

### constructor

• **new MultiCounter**()

## Methods

### free

▸ **free**(): `void`

#### Returns

`void`

#### Defined in

utils/lib/utils_metrics.d.ts:124

___

### get

▸ **get**(`label_values`): `bigint`

#### Parameters

| Name | Type |
| :------ | :------ |
| `label_values` | `string`[] |

#### Returns

`bigint`

#### Defined in

utils/lib/utils_metrics.d.ts:143

___

### increment

▸ **increment**(`label_values`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `label_values` | `string`[] |

#### Returns

`void`

#### Defined in

utils/lib/utils_metrics.d.ts:138

___

### increment\_by

▸ **increment_by**(`label_values`, `by`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `label_values` | `string`[] |
| `by` | `bigint` |

#### Returns

`void`

#### Defined in

utils/lib/utils_metrics.d.ts:134

___

### name

▸ **name**(): `string`

Returns the name of the counter vector given at construction.

#### Returns

`string`

#### Defined in

utils/lib/utils_metrics.d.ts:129
