import assert from 'assert'
import { u8aXOR } from './xor'

describe('testing XORing Uint8Array', function () {
  it('should XOR two arrays', function () {
    let a = new Uint8Array([0, 255, 0, 255, 0])
    let b = new Uint8Array([255, 0, 255, 0, 255])

    let aXORb = new Uint8Array([255, 255, 255, 255, 255])

    assert.deepEqual(u8aXOR(false, a, b), aXORb)

    u8aXOR(true, a, b)
    assert.deepEqual(a, aXORb)
  })

  it('should XOR more than two arrays', function () {
    let a = new Uint8Array([0, 255, 0, 255, 0])
    let b = new Uint8Array([255, 0, 255, 0, 255])
    let c = new Uint8Array([0, 0, 255, 0, 0])

    let aXORbXORc = new Uint8Array([255, 255, 0, 255, 255])

    assert.deepEqual(u8aXOR(false, a, b, c), aXORbXORc)

    u8aXOR(true, a, b, c)
    assert.deepEqual(a, aXORbXORc)
  })
})
