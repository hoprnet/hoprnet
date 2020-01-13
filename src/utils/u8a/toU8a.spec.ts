import assert from 'assert'
import { toU8a } from './toU8a'

describe('test number to u8a', function() {
  it('should return a u8a', function() {
    assert.deepEqual(toU8a(0), new Uint8Array([0x00]))

    assert.deepEqual(toU8a(1), new Uint8Array([0x01]))

    assert.deepEqual(toU8a(1234), new Uint8Array([0x04, 0xd2]))

    assert.deepEqual(toU8a(12345), new Uint8Array([0x30, 0x39]))

    assert.throws(() => toU8a(-1))

    assert.throws(() => toU8a(NaN))

    assert.throws(() => toU8a(Infinity))
  })

  it('should return a fixed-size u8a', function() {
    assert.deepEqual(toU8a(0, 1), new Uint8Array([0x00]))

    assert.deepEqual(toU8a(1, 1), new Uint8Array([0x01]))

    assert.deepEqual(toU8a(1234, 2), new Uint8Array([0x04, 0xd2]))

    assert.deepEqual(toU8a(12345, 2), new Uint8Array([0x30, 0x39]))

    assert.throws(() => toU8a(-1, 123))

    assert.throws(() => toU8a(NaN, 1234))

    assert.throws(() => toU8a(Infinity, 12345))

    assert.throws(() => toU8a(12345, 1))

    assert.deepEqual(toU8a(1, 2), new Uint8Array([0x00, 0x01]))

    assert.deepEqual(toU8a(1, 3), new Uint8Array([0x00, 0x00, 0x01]))

  })
})
