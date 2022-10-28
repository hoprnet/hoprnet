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

___

### free

▸ **free**(): `void`

#### Returns

`void`

___

### get\_sample\_count

▸ **get_sample_count**(): `bigint`

#### Returns

`bigint`

___

### get\_sample\_sum

▸ **get_sample_sum**(): `number`

#### Returns

`number`

___

### name

▸ **name**(): `string`

#### Returns

`string`

___

### observe

▸ **observe**(`value`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `value` | `number` |

#### Returns

`void`

___

### record\_measure

▸ **record_measure**(`timer`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `timer` | [`SimpleTimer`](SimpleTimer.md) |

#### Returns

`void`

___

### start\_measure

▸ **start_measure**(): [`SimpleTimer`](SimpleTimer.md)

#### Returns

[`SimpleTimer`](SimpleTimer.md)
