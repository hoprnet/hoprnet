[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / MultiGauge

# Class: MultiGauge

## Table of contents

### Constructors

- [constructor](MultiGauge.md#constructor)

### Methods

- [decrement](MultiGauge.md#decrement)
- [decrement\_by](MultiGauge.md#decrement_by)
- [free](MultiGauge.md#free)
- [get](MultiGauge.md#get)
- [increment](MultiGauge.md#increment)
- [increment\_by](MultiGauge.md#increment_by)
- [labels](MultiGauge.md#labels)
- [name](MultiGauge.md#name)
- [set](MultiGauge.md#set)

## Constructors

### constructor

• **new MultiGauge**()

## Methods

### decrement

▸ **decrement**(`label_values`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `label_values` | `string`[] |

#### Returns

`void`

#### Defined in

lib/utils_metrics.d.ts:164

___

### decrement\_by

▸ **decrement_by**(`label_values`, `by`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `label_values` | `string`[] |
| `by` | `number` |

#### Returns

`void`

#### Defined in

lib/utils_metrics.d.ts:160

___

### free

▸ **free**(): `void`

#### Returns

`void`

#### Defined in

lib/utils_metrics.d.ts:146

___

### get

▸ **get**(`label_values`): `number`

#### Parameters

| Name | Type |
| :------ | :------ |
| `label_values` | `string`[] |

#### Returns

`number`

#### Defined in

lib/utils_metrics.d.ts:174

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

lib/utils_metrics.d.ts:155

___

### increment\_by

▸ **increment_by**(`label_values`, `by`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `label_values` | `string`[] |
| `by` | `number` |

#### Returns

`void`

#### Defined in

lib/utils_metrics.d.ts:151

___

### labels

▸ **labels**(): `string`[]

#### Returns

`string`[]

#### Defined in

lib/utils_metrics.d.ts:182

___

### name

▸ **name**(): `string`

#### Returns

`string`

#### Defined in

lib/utils_metrics.d.ts:178

___

### set

▸ **set**(`label_values`, `value`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `label_values` | `string`[] |
| `value` | `number` |

#### Returns

`void`

#### Defined in

lib/utils_metrics.d.ts:169
