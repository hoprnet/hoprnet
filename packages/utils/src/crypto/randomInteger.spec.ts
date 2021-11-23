import assert from 'assert'
import { randomInteger, randomBigInteger } from './randomInteger'

describe('testing random-number generator', function () {
  let ATTEMPTS = 10000
  it(`should output generate values between [0, end)`, function () {
    let result: number
    let end = 10024
    for (let i = 0; i < ATTEMPTS; i++) {
      result = randomInteger(end)

      assert(0 <= result, result + ' gte 0')
      assert(result < end, result + ' lt ' + end)
    }
  })

  it(`should output generate bigint values between [0, end)`, function () {
    let result: bigint
    let end = BigInt(Number.MAX_SAFE_INTEGER) + 10024n
    for (let i = 0; i < ATTEMPTS; i++) {
      result = randomBigInteger(end)

      assert(0n <= result, result + ' gte 0')
      assert(result < end, result + ' lt ' + end)
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

  it(`should output bigint values between [start, end) with start > 0`, function () {
    let result: bigint
    let start = BigInt(Number.MAX_SAFE_INTEGER) + 253n
    let end = BigInt(Number.MAX_SAFE_INTEGER) + 73111n
    for (let i = 0; i < ATTEMPTS; i++) {
      result = randomBigInteger(start, end)

      assert(start <= result && result < end)
    }
  })

  it('should yield correct values for edge cases', function () {
    assert(randomInteger(23, 24) == 23)

    assert(randomInteger(1) == 0)

    assert.throws(() => randomInteger(-1), Error(`invalid range`))

    // Number.MAX_SAFE_INTEGER * 2 is strictly greater than Number.MAX_SAFE_INTEGER
    // due to increased exponent in IEEE754 representation
    assert.throws(() => randomInteger(0, Number.MAX_SAFE_INTEGER * 2), Error(`invalid range`))
  })

  it('should yield correct values for bigint edge cases', function () {
    assert(
      randomBigInteger(BigInt(Number.MAX_SAFE_INTEGER) + 23n, BigInt(Number.MAX_SAFE_INTEGER) + 24n) ==
        BigInt(Number.MAX_SAFE_INTEGER) + 23n
    )

    assert(randomBigInteger(1n) == 0n)

    assert.throws(() => randomBigInteger(-1n), Error(`invalid range`))
  })
})
