import assert from 'assert'
import {toLengthPrefixedU8a} from './toLengthPrefixedU8a'
import {u8aConcat} from './concat'

describe('test u8a to length-prefixed u8a', function () {
  it('should return a length-prefixed u8a', function () {
    assert.deepEqual(toLengthPrefixedU8a(new Uint8Array([1])), new Uint8Array([0, 0, 0, 1, 1]))

    assert.deepEqual(
      toLengthPrefixedU8a(new Uint8Array(256)),
      u8aConcat(new Uint8Array([0, 0, 1, 0]), new Uint8Array(256))
    )

    assert.throws(() => toLengthPrefixedU8a(new Uint8Array([1]), undefined, 2))

    assert.deepEqual(toLengthPrefixedU8a(new Uint8Array([1]), undefined, 6), new Uint8Array([0, 0, 0, 1, 1, 0]))
  })

  it('should return a length-prefixed u8a with additional padding', function () {
    assert.deepEqual(toLengthPrefixedU8a(new Uint8Array([1]), new Uint8Array([1])), new Uint8Array([0, 0, 0, 1, 1, 1]))

    assert.deepEqual(
      toLengthPrefixedU8a(new Uint8Array(256), new Uint8Array([1])),
      u8aConcat(new Uint8Array([0, 0, 1, 0]), new Uint8Array([1]), new Uint8Array(256))
    )

    assert.throws(() => toLengthPrefixedU8a(new Uint8Array([1]), new Uint8Array([1]), 5))

    assert.deepEqual(
      toLengthPrefixedU8a(new Uint8Array([1]), new Uint8Array([1]), 7),
      new Uint8Array([0, 0, 0, 1, 1, 1, 0])
    )
  })
})
