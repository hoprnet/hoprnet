import assert from 'assert'
import { randomInteger } from './randomInteger'

describe('testing random-number generator', function () {
  let ATTEMPTS = 10000
  it(`should output generate values between [0, end)`, function () {
    let result: number
    let end = 10024
    for (let i = 0; i < ATTEMPTS; i++) {
      result = randomInteger(end)

      assert(0 <= result && result < end)
    }
  })

  it(`should output values between [start, end) with start > 0`, function () {
    let result: number
    let start = 253
    let end = 73111
    for (let i = 0; i < ATTEMPTS; i++) {
      result = randomInteger(start, end)

      assert(start <= result && result < end)
    }
  })

  it('should throw error for falsy interval input', function () {
    assert.throws(() => randomInteger(2, 1))

    assert.throws(() => randomInteger(Math.pow(2, 32)))

    assert.throws(() => randomInteger(-1))

    assert.throws(() => randomInteger(-1, -2))
  })
})
