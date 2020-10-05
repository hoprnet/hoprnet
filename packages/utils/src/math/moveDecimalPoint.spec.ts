import assert from 'assert'
import { moveDecimalPoint } from './moveDecimalPoint'

describe('test moveDecimalPoint', () => {
  it('should result to 100', () => {
    assert.equal(moveDecimalPoint(1, 2), '100', 'check moveDecimalPoint')
  })
  it('should result to 100', () => {
    assert.equal(moveDecimalPoint(0.01, 4), '100', 'check moveDecimalPoint')
  })

  it('should result to 0.01', () => {
    assert.equal(moveDecimalPoint(1, -2), '0.01', 'check moveDecimalPoint')
  })
  it('should result to 0.01', () => {
    assert.equal(moveDecimalPoint(100, -4), '0.01', 'check moveDecimalPoint')
  })
})
