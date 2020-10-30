import assert from 'assert'
import {randomSubset} from './randomSubset'

const SET_SIZE = 50
const SUBSET_SIZE = 20

describe('testing random subset', function () {
  it('should return a subset with a filter function', function () {
    assert.deepEqual(randomSubset([1], 1), [1])

    assert.deepEqual(randomSubset([1, 2, 3], 3).sort(), [1, 2, 3])

    let array = []

    for (let i = 0; i < SET_SIZE; i++) {
      array.push(i)
    }

    let result = randomSubset(array, SUBSET_SIZE, (value: number) => value % 2 == 0)

    assert(result.length == SUBSET_SIZE)

    let set = new Set<number>()
    result.forEach((value) => {
      assert(0 <= value && value < SET_SIZE && value % 2 == 0)
      assert(!set.has(value))
      set.add(value)
    })

    let orderedSet = new Set<number>(result.slice().sort())

    let i = 0
    let notEqualFound = false
    orderedSet.forEach((value: number) => {
      notEqualFound = notEqualFound || result[i++] != value
    })

    assert(
      notEqualFound,
      `Elements should be unordered with very high probability. (This test might fail once in a while)`
    )
  })

  it('should return a subset', function () {
    assert.deepEqual(
      randomSubset([1, 2], 1, (value: number) => value == 1),
      [1]
    )

    assert.deepEqual(randomSubset([1, 2, 3], 3, (value: number) => [1, 2, 3].includes(value)).sort(), [1, 2, 3])

    let array = []

    for (let i = 0; i < SET_SIZE; i++) {
      array.push(i)
    }

    let result = randomSubset(array, SUBSET_SIZE)

    assert(result.length == SUBSET_SIZE)

    let set = new Set<number>()
    result.forEach((value) => {
      assert(0 <= value && value < SET_SIZE)
      assert(!set.has(value))
      set.add(value)
    })

    let orderedSet = new Set<number>(result.slice().sort())

    let i = 0
    let notEqualFound = false
    orderedSet.forEach((value: number) => {
      notEqualFound = notEqualFound || result[i++] != value
    })

    assert(
      notEqualFound,
      `Elements should be unordered with very high probability. (This test might fail once in a while)`
    )
  })
})
