[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / SimpleHistogram

# Class: SimpleHistogram

Represents a histogram with floating point values.
Wrapper for Histogram type

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

Stops the given timer and discards the measured duration in seconds and returns it.

#### Parameters

| Name | Type |
| :------ | :------ |
| `timer` | [`SimpleTimer`](SimpleTimer.md) |

#### Returns

`number`

#### Defined in

utils/lib/utils_metrics.d.ts:312

___

### free

▸ **free**(): `void`

#### Returns

`void`

#### Defined in

utils/lib/utils_metrics.d.ts:296

___

### get\_sample\_count

▸ **get_sample_count**(): `bigint`

Get all samples count

#### Returns

`bigint`

#### Defined in

utils/lib/utils_metrics.d.ts:317

___

### get\_sample\_sum

▸ **get_sample_sum**(): `number`

Get all samples sum

#### Returns

`number`

#### Defined in

utils/lib/utils_metrics.d.ts:322

___

### name

▸ **name**(): `string`

Returns the name of the histogram given at construction.

#### Returns

`string`

#### Defined in

utils/lib/utils_metrics.d.ts:327

___

### observe

▸ **observe**(`value`): `void`

Records a value observation to the histogram.

#### Parameters

| Name | Type |
| :------ | :------ |
| `value` | `number` |

#### Returns

`void`

#### Defined in

utils/lib/utils_metrics.d.ts:301

___

### record\_measure

▸ **record_measure**(`timer`): `void`

Stops the given timer and records the elapsed duration in seconds to the histogram.

#### Parameters

| Name | Type |
| :------ | :------ |
| `timer` | [`SimpleTimer`](SimpleTimer.md) |

#### Returns

`void`

#### Defined in

utils/lib/utils_metrics.d.ts:306

___

### start\_measure

▸ **start_measure**(): [`SimpleTimer`](SimpleTimer.md)

#### Returns

[`SimpleTimer`](SimpleTimer.md)

#### Defined in

utils/lib/utils_metrics.d.ts:331
