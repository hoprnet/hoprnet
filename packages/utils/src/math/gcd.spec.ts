import assert from 'assert'
import {gcd} from './gcd'

describe('check gcd computation', function () {
  it('should compute the gcd of two integers', function () {
    assert(gcd(0, 0) == 1, 'Gcd of 0 should be 1')
    assert(gcd(1, 1) == 1, 'Gcd of 1 and 1 should be 1')

    assert(gcd(23, 31) == 1, 'Gcd of two primes should be 1')

    assert(gcd(8, 4) == 4)

    assert(gcd(23, 23) == 23)
  })
})
