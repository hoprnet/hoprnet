[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / MultiCounter

# Class: MultiCounter

## Table of contents

### Constructors

- [constructor](MultiCounter.md#constructor)

### Methods

- [free](MultiCounter.md#free)
- [get](MultiCounter.md#get)
- [increment](MultiCounter.md#increment)
- [increment\_by](MultiCounter.md#increment_by)
- [labels](MultiCounter.md#labels)
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

lib/utils_metrics.d.ts:81

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

lib/utils_metrics.d.ts:95

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

lib/utils_metrics.d.ts:90

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

lib/utils_metrics.d.ts:86

___

### labels

▸ **labels**(): `string`[]

#### Returns

`string`[]

#### Defined in

lib/utils_metrics.d.ts:103

___

### name

▸ **name**(): `string`

#### Returns

`string`

#### Defined in

lib/utils_metrics.d.ts:99
