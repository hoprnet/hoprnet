import { Response } from './response'
import { stringToU8a, u8aAdd, toU8a } from '../u8a'
import assert from 'assert'

describe(`test response`, function () {
  const FIELD_ORDER = stringToU8a('0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141')
  const INVALID_FIELD_ELEMENT = u8aAdd(false, FIELD_ORDER, toU8a(1, 32))

  it('check response generation and edge cases', function () {
      // Zero is not allowed
    assert.throws(() => new Response(toU8a(0, 32)))

    // FIELD_ORDER is in same equality class as zero, hence not allowed
    assert.throws(() => new Response(FIELD_ORDER))

    // INVALID_FIELD_ELEMENT is outside field, hence not allowed
    assert.throws(() => new Response(INVALID_FIELD_ELEMENT))
  })
})
