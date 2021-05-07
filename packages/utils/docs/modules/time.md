[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / time

# Module: time

## Table of contents

### Variables

- [durations](time.md#durations)

### Functions

- [isExpired](time.md#isexpired)

## Variables

### durations

• `Const` **durations**: _object_

#### Type declaration

| Name      | Type                              |
| :-------- | :-------------------------------- |
| `days`    | (`days`: _number_) => _number_    |
| `hours`   | (`hours`: _number_) => _number_   |
| `minutes` | (`minutes`: _number_) => _number_ |
| `seconds` | (`seconds`: _number_) => _number_ |

Defined in: [time.ts:1](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/time.ts#L1)

## Functions

### isExpired

▸ **isExpired**(`value`: _number_, `now`: _number_, `ttl`: _number_): _boolean_

Compares timestamps to find out if "value" has expired.

#### Parameters

| Name    | Type     | Description                               |
| :------ | :------- | :---------------------------------------- |
| `value` | _number_ | timestamp to compare with                 |
| `now`   | _number_ | timestamp example: `new Date().getTime()` |
| `ttl`   | _number_ | in milliseconds                           |

**Returns:** _boolean_

true if it's expired

Defined in: [time.ts:23](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/time.ts#L23)
