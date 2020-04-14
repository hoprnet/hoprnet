import assert from 'assert'
import { toDecimalPoint } from './toDecimalPoint'

describe('test toDecimalPoint', function () {
  it('should result to 100', function () {
    assert.equal(toDecimalPoint(1, 2), '100', 'check toDecimalPoint')
  })
  it('should result to 100', function () {
    assert.equal(toDecimalPoint(0.01, 4), '100', 'check toDecimalPoint')
  })

  it('should result to 0.01', function () {
    assert.equal(toDecimalPoint(1, -2), '0.01', 'check toDecimalPoint')
  })
  it('should result to 0.01', function () {
    assert.equal(toDecimalPoint(100, -4), '0.01', 'check toDecimalPoint')
  })
})
