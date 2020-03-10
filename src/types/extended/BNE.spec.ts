import assert from 'assert'
import BNE from './BNE'

describe('test BNE', function() {
  it('should returns a Uint8Array', function() {
    const number = 1

    assert.deepEqual(new BNE(number).toU8a(), new Uint8Array([number]), 'check if BNE u8a array is correct')
  })
})
