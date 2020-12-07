import assert from 'assert'
import { randomInteger } from './randomInteger'
import { u8aAdd } from './u8a'

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

    assert.throws(() => randomInteger(2 ** 32))

    assert.throws(() => randomInteger(-1))

    assert.throws(() => randomInteger(-1, -2))
  })

  it('should yield correct values for edge cases', function () {
    const MAX_INTEGER = 2 ** 31

    assert(randomInteger(0, MAX_INTEGER, new Uint8Array(4).fill(0xff)) == MAX_INTEGER - 1)

    assert.throws(() => randomInteger(0, MAX_INTEGER + 1, new Uint8Array(4).fill(0xff)))

    assert(randomInteger(0, 2 ** 24, new Uint8Array(4).fill(0xff)) == 2 ** 24 - 1)

    assert(randomInteger(0, 1) == 0)

    assert.throws(() => randomInteger(0))

    assert(randomInteger(23, 24) == 23)
  })

  it('should verify the randomInteger by using deterministic seeds', function () {
    const LENGTH = 2

    const LOWERBOUND = 27
    const UPPERBOUND = 480
    const bytes = new Uint8Array(LENGTH).fill(0)

    const ONE = new Uint8Array(LENGTH).fill(0)
    ONE[ONE.length - 1] = 1

    for (let i = 0; i < ATTEMPTS; i++) {
      u8aAdd(true, bytes, ONE)

      let result = randomInteger(LOWERBOUND, UPPERBOUND, bytes)
      assert(result < UPPERBOUND && result >= LOWERBOUND)
    }
  })
})
