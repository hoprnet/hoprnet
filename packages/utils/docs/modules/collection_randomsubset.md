[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / collection/randomSubset

# Module: collection/randomSubset

## Table of contents

### Functions

- [randomSubset](collection_randomsubset.md#randomsubset)

## Functions

### randomSubset

▸ **randomSubset**<T\>(`array`: T[], `subsetSize`: _number_, `filter?`: (`candidate`: T) => _boolean_): T[]

Picks @param subsetSize elements at random from @param array .
The order of the picked elements does not coincide with their
order in @param array

**`notice`** If less than @param subsetSize elements pass the test,
the result will contain less than @param subsetSize elements.

#### Type parameters

| Name |
| :--- |
| `T`  |

#### Parameters

| Name         | Type                          | Description                                                                                   |
| :----------- | :---------------------------- | :-------------------------------------------------------------------------------------------- |
| `array`      | T[]                           | the array to pick the elements from                                                           |
| `subsetSize` | _number_                      | the requested size of the subset                                                              |
| `filter?`    | (`candidate`: T) => _boolean_ | called with `(peerInfo)` and should return `true` for every node that should be in the subset |

**Returns:** T[]

array with at most @param subsetSize elements
that pass the test.

Defined in: [collection/randomSubset.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/collection/randomSubset.ts#L20)
