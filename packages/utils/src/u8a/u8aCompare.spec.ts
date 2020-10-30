import assert from 'assert'
import {u8aCompare, A_EQUALS_B, A_STRICLY_LESS_THAN_B, A_STRICTLY_GREATER_THAN_B} from './u8aCompare'

describe('test u8aCompare', function () {
  it('should compare uint8Arrays', function () {
    assert(u8aCompare(new Uint8Array([0]), new Uint8Array([1])) == A_STRICLY_LESS_THAN_B)

    assert(u8aCompare(new Uint8Array([0]), new Uint8Array([0])) == A_EQUALS_B)

    assert(u8aCompare(new Uint8Array([1]), new Uint8Array([0])) == A_STRICTLY_GREATER_THAN_B)

    assert.throws(() => u8aCompare(new Uint8Array([0, 0]), new Uint8Array([0])))

    assert(u8aCompare(new Uint8Array([0, 0]), new Uint8Array([0, 1])) == A_STRICLY_LESS_THAN_B)

    assert(u8aCompare(new Uint8Array([1, 0]), new Uint8Array([0, 1])) == A_STRICTLY_GREATER_THAN_B)

    assert(u8aCompare(new Uint8Array([0, 0]), new Uint8Array([0, 0])) == A_EQUALS_B)
  })
})
