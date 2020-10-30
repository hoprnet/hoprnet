import {u8aToHex} from './toHex'
import assert from 'assert'

describe('test toHex', function () {
  it('should create a Hex string', function () {
    assert(u8aToHex(new Uint8Array([])) == '0x')

    assert(u8aToHex(new Uint8Array([]), false) == '')

    assert(u8aToHex(new Uint8Array([1, 2, 3])) == '0x010203')

    assert(u8aToHex(new Uint8Array([1, 2, 3]), false) == '010203')
  })
})
