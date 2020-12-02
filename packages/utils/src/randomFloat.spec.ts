import { randomFloat } from './randomFloat'
import assert from 'assert'

describe('test randomFloat', function () {
  let ATTEMPTS = 10000
  it('should output a random float', function () {
    for (let i = 0; i < ATTEMPTS; i++) {
      let result = randomFloat()

      assert(0 <= result && result < 1)
    }
  })
})
