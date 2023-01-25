[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / SimpleHistogram

# Class: SimpleHistogram

## Table of contents

### Constructors

- [constructor](SimpleHistogram.md#constructor)

### Methods

- [cancel\_measure](SimpleHistogram.md#cancel_measure)
- [free](SimpleHistogram.md#free)
- [get\_sample\_count](SimpleHistogram.md#get_sample_count)
- [get\_sample\_sum](SimpleHistogram.md#get_sample_sum)
- [name](SimpleHistogram.md#name)
- [observe](SimpleHistogram.md#observe)
- [record\_measure](SimpleHistogram.md#record_measure)
- [start\_measure](SimpleHistogram.md#start_measure)

## Constructors

### constructor

• **new SimpleHistogram**()

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

lib/utils_metrics.d.ts:259

___

### free

▸ **free**(): `void`

#### Returns

`void`

#### Defined in

lib/utils_metrics.d.ts:242

___

### get\_sample\_count

▸ **get_sample_count**(): `bigint`

#### Returns

`bigint`

#### Defined in

lib/utils_metrics.d.ts:263

___

### get\_sample\_sum

▸ **get_sample_sum**(): `number`

#### Returns

`number`

#### Defined in

lib/utils_metrics.d.ts:267

___

### name

▸ **name**(): `string`

#### Returns

`string`

#### Defined in

lib/utils_metrics.d.ts:271

___

### observe

▸ **observe**(`value`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `value` | `number` |

#### Returns

`void`

#### Defined in

lib/utils_metrics.d.ts:246

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

lib/utils_metrics.d.ts:254

___

### start\_measure

▸ **start_measure**(): [`SimpleTimer`](SimpleTimer.md)

#### Returns

[`SimpleTimer`](SimpleTimer.md)

#### Defined in

lib/utils_metrics.d.ts:250
