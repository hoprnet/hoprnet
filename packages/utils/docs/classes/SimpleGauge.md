[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / SimpleGauge

# Class: SimpleGauge

Represents a simple gauge with floating point values.
Wrapper for Gauge type

## Table of contents

### Constructors

- [constructor](SimpleGauge.md#constructor)

### Methods

- [decrement](SimpleGauge.md#decrement)
- [free](SimpleGauge.md#free)
- [get](SimpleGauge.md#get)
- [increment](SimpleGauge.md#increment)
- [name](SimpleGauge.md#name)
- [set](SimpleGauge.md#set)

## Constructors

### constructor

• **new SimpleGauge**()

## Methods

### decrement

▸ **decrement**(`by`): `void`

Decrements the gauge by the given value.

#### Parameters

| Name | Type |
| :------ | :------ |
| `by` | `number` |

#### Returns

`void`

#### Defined in

utils/lib/utils_metrics.d.ts:274

___

### free

▸ **free**(): `void`

#### Returns

`void`

#### Defined in

utils/lib/utils_metrics.d.ts:264

___

### get

▸ **get**(): `number`

Retrieves the value of the gauge

#### Returns

`number`

#### Defined in

utils/lib/utils_metrics.d.ts:284

___

### increment

▸ **increment**(`by`): `void`

Increments the gauge by the given value.

#### Parameters

| Name | Type |
| :------ | :------ |
| `by` | `number` |

#### Returns

`void`

#### Defined in

utils/lib/utils_metrics.d.ts:269

___

### name

▸ **name**(): `string`

Returns the name of the gauge given at construction.

#### Returns

`string`

#### Defined in

utils/lib/utils_metrics.d.ts:289

___

### set

▸ **set**(`value`): `void`

Sets the gauge to the given value.

#### Parameters

| Name | Type |
| :------ | :------ |
| `value` | `number` |

#### Returns

`void`

#### Defined in

utils/lib/utils_metrics.d.ts:279
