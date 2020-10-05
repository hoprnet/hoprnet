import assert from 'assert'
import { randomPermutation } from './randomPermutation'

describe('testing random permutation', function () {
  let ATTEMPTS = 2

  it(`should apply a random permutation`, function () {
    for (let counter = 0; counter < ATTEMPTS; counter++) {
      let array = []
      for (let i = 0; i < 30; i++) {
        array.push(i)
      }

      let length = array.length
      randomPermutation(array)

      assert(array.length == length)

      let set = new Set<number>()
      array.forEach((value: number) => {
        assert(!set.has(value))
        set.add(value)
      })
    }
  })
})
