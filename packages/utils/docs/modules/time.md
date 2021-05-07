[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / time

# Module: time

## Table of contents

### Variables

- [durations](time.md#durations)

### Functions

- [isExpired](time.md#isexpired)

## Variables

### durations

• `Const` **durations**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `days` | (`days`: *number*) => *number* |
| `hours` | (`hours`: *number*) => *number* |
| `minutes` | (`minutes`: *number*) => *number* |
| `seconds` | (`seconds`: *number*) => *number* |

Defined in: [time.ts:1](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/time.ts#L1)

## Functions

### isExpired

▸ **isExpired**(`value`: *number*, `now`: *number*, `ttl`: *number*): *boolean*

Compares timestamps to find out if "value" has expired.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `value` | *number* | timestamp to compare with |
| `now` | *number* | timestamp example: `new Date().getTime()` |
| `ttl` | *number* | in milliseconds |

**Returns:** *boolean*

true if it's expired

Defined in: [time.ts:23](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/time.ts#L23)
