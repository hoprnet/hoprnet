import assert from 'assert'
import * as utils from '.'

describe('test utils', function () {
  it('should compute a winning probability and convert it to float', function () {
    for (let i = 0; i < 10; i++) {
      let prob = Math.random()

      let winProb = utils.computeWinningProbability(prob)

      assert(Math.abs(prob - utils.getWinProbabilityAsFloat(winProb)) <= 0.0001)
    }
  })
})
