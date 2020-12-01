import assert from 'assert'
import { lengthPrefixedToU8a } from './lengthPrefixedToU8a'
import { toLengthPrefixedU8a } from './toLengthPrefixedU8a'

describe('test length-prefixed to u8a', function () {
  it('should convert a length-prefixed u8a to u8a', function () {
    assert.deepStrictEqual(lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1, 255])), new Uint8Array([255]))

    assert.throws(() => lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 255, 1])))

    assert.throws(() => lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1])))

    assert.throws(() => lengthPrefixedToU8a(new Uint8Array([0, 0, 0])))

    assert.deepEqual(
      lengthPrefixedToU8a(toLengthPrefixedU8a(new Uint8Array([1, 2, 3, 4]))),
      new Uint8Array([1, 2, 3, 4])
    )

    assert.throws(() => lengthPrefixedToU8a(new Uint8Array([]), undefined, 1))

    assert.deepEqual(lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1, 1, 0]), undefined, 6), new Uint8Array([1]))
  })

  it('should convert a length-prefixed u8a with additional padding to u8a', function () {
    assert.deepStrictEqual(
      lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1, 1, 255]), new Uint8Array([1])),
      new Uint8Array([255])
    )

    assert.throws(() => lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1, 1, 255]), new Uint8Array([2])))

    assert.throws(() => lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1, 255]), new Uint8Array([2])))

    assert.throws(() => lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 255, 1]), new Uint8Array([1])))

    assert.throws(() => lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1]), new Uint8Array([1])))

    assert.throws(() => lengthPrefixedToU8a(new Uint8Array([0, 0, 0]), new Uint8Array([1])))

    assert.deepStrictEqual(
      lengthPrefixedToU8a(toLengthPrefixedU8a(new Uint8Array([1, 2, 3, 4]), new Uint8Array([1])), new Uint8Array([1])),
      new Uint8Array([1, 2, 3, 4])
    )

    assert.throws(() => lengthPrefixedToU8a(new Uint8Array([]), new Uint8Array([1]), 2))

    assert.deepStrictEqual(
      lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1, 1, 1, 0]), undefined, 7),
      new Uint8Array([1])
    ),
      new Uint8Array([1])
  })
})
