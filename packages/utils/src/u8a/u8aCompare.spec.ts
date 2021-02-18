import assert from 'assert'
import {
  u8aCompare,
  u8aLessThanOrEqual,
  A_EQUALS_B,
  A_STRICLY_LESS_THAN_B,
  A_STRICTLY_GREATER_THAN_B
} from './u8aCompare'
import { toU8a } from './index'

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

describe('u8aLessThanOrEqual', function () {
  it('lt', function () {
    assert(u8aLessThanOrEqual(toU8a(1), toU8a(2)))
    assert(u8aLessThanOrEqual(toU8a(0), toU8a(2)))
  })

  it('eq', function () {
    assert(u8aLessThanOrEqual(toU8a(1), toU8a(1)))
    assert(u8aLessThanOrEqual(toU8a(0), toU8a(0)))
  })
  it('gt', function () {
    assert(!u8aLessThanOrEqual(toU8a(2), toU8a(1)))
    assert(!u8aLessThanOrEqual(toU8a(1), toU8a(0)))
  })
})
