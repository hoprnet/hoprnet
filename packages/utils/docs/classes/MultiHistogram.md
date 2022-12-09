[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / MultiHistogram

# Class: MultiHistogram

## Table of contents

### Constructors

- [constructor](MultiHistogram.md#constructor)

### Methods

- [cancel\_measure](MultiHistogram.md#cancel_measure)
- [free](MultiHistogram.md#free)
- [get\_sample\_count](MultiHistogram.md#get_sample_count)
- [get\_sample\_sum](MultiHistogram.md#get_sample_sum)
- [labels](MultiHistogram.md#labels)
- [name](MultiHistogram.md#name)
- [observe](MultiHistogram.md#observe)
- [record\_measure](MultiHistogram.md#record_measure)
- [start\_measure](MultiHistogram.md#start_measure)

## Constructors

### constructor

• **new MultiHistogram**()

## Methods

### cancel\_measure

▸ **cancel_measure**(`timer`): `number`

#### Parameters

| Name | Type |
| :------ | :------ |
| `timer` | [`SimpleTimer`](SimpleTimer.md) |

#### Returns

`number`

#### Defined in

lib/utils_metrics.d.ts:154

___

### free

▸ **free**(): `void`

#### Returns

`void`

#### Defined in

lib/utils_metrics.d.ts:135

___

### get\_sample\_count

▸ **get_sample_count**(`label_values`): `bigint`

#### Parameters

| Name | Type |
| :------ | :------ |
| `label_values` | `string`[] |

#### Returns

`bigint`

#### Defined in

lib/utils_metrics.d.ts:159

___

### get\_sample\_sum

▸ **get_sample_sum**(`label_values`): `number`

#### Parameters

| Name | Type |
| :------ | :------ |
| `label_values` | `string`[] |

#### Returns

`number`

#### Defined in

lib/utils_metrics.d.ts:164

___

### labels

▸ **labels**(): `string`[]

#### Returns

`string`[]

#### Defined in

lib/utils_metrics.d.ts:172

___

### name

▸ **name**(): `string`

#### Returns

`string`

#### Defined in

lib/utils_metrics.d.ts:168

___

### observe

▸ **observe**(`label_values`, `value`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `label_values` | `string`[] |
| `value` | `number` |

#### Returns

`void`

#### Defined in

lib/utils_metrics.d.ts:140

___

### record\_measure

▸ **record_measure**(`timer`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `timer` | [`SimpleTimer`](SimpleTimer.md) |

#### Returns

`void`

#### Defined in

lib/utils_metrics.d.ts:149

___

### start\_measure

▸ **start_measure**(`label_values`): [`SimpleTimer`](SimpleTimer.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `label_values` | `string`[] |

#### Returns

[`SimpleTimer`](SimpleTimer.md)

#### Defined in

lib/utils_metrics.d.ts:145
