[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / collection/promise-pool

# Module: collection/promise-pool

## Table of contents

### Functions

- [limitConcurrency](collection_promise_pool.md#limitconcurrency)

## Functions

### limitConcurrency

▸ **limitConcurrency**<T\>(`maxConcurrency`: _number_, `exitCond`: () => _boolean_, `createPromise`: () => _Promise_<T\>, `maxIterations?`: _number_): _Promise_<T[]\>

#### Type parameters

| Name |
| :--- |
| `T`  |

#### Parameters

| Name             | Type                | Default value |
| :--------------- | :------------------ | :------------ |
| `maxConcurrency` | _number_            | -             |
| `exitCond`       | () => _boolean_     | -             |
| `createPromise`  | () => _Promise_<T\> | -             |
| `maxIterations`  | _number_            | 1e3           |

**Returns:** _Promise_<T[]\>

Defined in: [collection/promise-pool.ts:1](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/collection/promise-pool.ts#L1)
