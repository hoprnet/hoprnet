[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / collection/randomPermutation

# Module: collection/randomPermutation

## Table of contents

### Functions

- [randomPermutation](collection_randompermutation.md#randompermutation)

## Functions

### randomPermutation

▸ **randomPermutation**<T\>(`array`: T[]): T[]

Return a random permutation of the given `array`
by using the (optimized) Fisher-Yates shuffling algorithm.

**`example`**

```javascript
randomPermutation([1, 2, 3, 4])
// first run: [2,4,1,2]
// second run: [3,1,2,4]
// ...
```

#### Type parameters

| Name |
| :--- |
| `T`  |

#### Parameters

| Name    | Type | Description            |
| :------ | :--- | :--------------------- |
| `array` | T[]  | the array to permutate |

**Returns:** T[]

Defined in: [collection/randomPermutation.ts:18](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/collection/randomPermutation.ts#L18)
