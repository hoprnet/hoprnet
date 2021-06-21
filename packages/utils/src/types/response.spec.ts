import { Response } from './response'
import { stringToU8a, u8aAdd, toU8a } from '../u8a'
import assert from 'assert'

describe(`test response`, function () {
  const FIELD_ORDER = stringToU8a('0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141')
  const INVALID_FIELD_ELEMENT = u8aAdd(false, FIELD_ORDER, toU8a(1, 32))

  it('check response generation and edge cases', function () {
    // Zero is not allowed
    assert.throws(
      () => new Response(toU8a(0, 32)),
      Error(`Invalid input argument. Given value is not a valid field element.`)
    )

    // FIELD_ORDER is in same equality class as zero, hence not allowed
    assert.throws(
      () => new Response(FIELD_ORDER),
      Error(`Invalid input argument. Given value is not a valid field element.`)
    )

    // INVALID_FIELD_ELEMENT is outside of the field, hence not allowed
    assert.throws(
      () => new Response(INVALID_FIELD_ELEMENT),
      Error(`Invalid input argument. Given value is not a valid field element.`)
    )
  })
})
