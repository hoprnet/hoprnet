import assert from 'assert'
import Uint8ArrayE from './uint8ArrayE'

describe('test Uint8ArrayE', function () {
  const arr = new Uint8ArrayE([1, 2, 3, 4, 5])
  const hex = '0x0102030405'

  it('should return an equal Uint8Array', function () {
    const r = arr.toU8a()

    assert.deepEqual(arr, r, 'check if Uint8Array is correct')
  })

  it('should return a hex', function () {
    const r = arr.toHex()

    assert.deepEqual(hex, r, 'check if hex is correct')
  })

  it('should return a subarray', function () {
    const r = arr.subarray(1, 3)

    assert.deepEqual(new Uint8Array([2, 3]), r, 'check if subarray is correct')
  })
})
