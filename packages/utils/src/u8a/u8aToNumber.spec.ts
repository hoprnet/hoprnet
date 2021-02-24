import assert from 'assert'
import { u8aToNumber } from './u8aToNumber'

describe('test u8aToNumber', function () {
  it('should convert a u8a to a number', function () {
    assert(u8aToNumber(new Uint8Array()) == 0)

    assert(u8aToNumber(new Uint8Array([1])) == 1)

    assert(u8aToNumber(new Uint8Array([1, 0])) == 256)

    assert(u8aToNumber(new Uint8Array([1, 1])) == 257)
  })
})
