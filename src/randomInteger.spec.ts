import assert from 'assert'
import { randomInteger } from './randomInteger'

describe('testing random-number generator', function () {
  let ATTEMPTS = 100
  it(`should output values between '0' and '23'`, function () {
    let result: number
    for (let i = 0; i < ATTEMPTS; i++) {
      result = randomInteger(23)

      assert(0 <= result && result < 23)
    }
  })

  it(`should output values between '31' and '61'`, function () {
    let result: number
    for (let i = 0; i < ATTEMPTS; i++) {
      result = randomInteger(31, 61)

      assert(31 <= result && result < 61)
    }
  })

  it(`should output values between '0' and '8'`, function () {
    let result: number
    for (let i = 0; i < ATTEMPTS; i++) {
      result = randomInteger(0, 8)

      assert(0 <= result && result < 8)
    }
  })

  it(`should output values between '23' and '7500000'`, function () {
    let result: number
    for (let i = 0; i < ATTEMPTS; i++) {
      result = randomInteger(23, 7500000)

      assert(23 <= result && result < 7500000)
    }
  })

  it('should throw error for falsy interval input', function () {
    assert.throws(() => randomInteger(2, 1))

    assert.throws(() => randomInteger(Math.pow(2, 32)))

    assert.throws(() => randomInteger(-1))

    assert.throws(() => randomInteger(-1, -2))
  })
})
