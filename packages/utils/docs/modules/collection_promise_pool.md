[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / collection/promise-pool

# Module: collection/promise-pool

## Table of contents

### Functions

- [limitConcurrency](collection_promise_pool.md#limitconcurrency)

## Functions

### limitConcurrency

▸ **limitConcurrency**<T\>(`maxConcurrency`: *number*, `exitCond`: () => *boolean*, `createPromise`: () => *Promise*<T\>, `maxIterations?`: *number*): *Promise*<T[]\>

#### Type parameters

| Name |
| :------ |
| `T` |

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `maxConcurrency` | *number* | - |
| `exitCond` | () => *boolean* | - |
| `createPromise` | () => *Promise*<T\> | - |
| `maxIterations` | *number* | 1e3 |

**Returns:** *Promise*<T[]\>

Defined in: [collection/promise-pool.ts:1](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/collection/promise-pool.ts#L1)
