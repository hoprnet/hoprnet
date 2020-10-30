import assert from 'assert'
import { moveDecimalPoint } from './moveDecimalPoint'

describe('test moveDecimalPoint', function () {
  it('should result to 100', function () {
    assert.equal(moveDecimalPoint(1, 2), '100', 'check moveDecimalPoint')
  })
  it('should result to 100', function () {
    assert.equal(moveDecimalPoint(0.01, 4), '100', 'check moveDecimalPoint')
  })

  it('should result to 0.01', function () {
    assert.equal(moveDecimalPoint(1, -2), '0.01', 'check moveDecimalPoint')
  })
  it('should result to 0.01', function () {
    assert.equal(moveDecimalPoint(100, -4), '0.01', 'check moveDecimalPoint')
  })
})
