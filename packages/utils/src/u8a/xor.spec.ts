import assert from 'assert'
import { u8aXOR } from './xor'

describe('testing XORing Uint8Array', function () {
  it('should XOR two arrays', function () {
    let a = new Uint8Array([0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0])
    let b = new Uint8Array([255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255])

    let aXORb = new Uint8Array(15).fill(255)

    assert.deepStrictEqual(u8aXOR(false, a, b), aXORb)

    u8aXOR(true, a, b)
    assert.deepStrictEqual(a, aXORb)
  })

  it('should XOR more than two arrays', function () {
    let a = new Uint8Array([0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0])
    let b = new Uint8Array([255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255])
    let c = new Uint8Array(15).fill(15)

    let aXORbXORc = new Uint8Array(15).fill(240)

    assert.deepStrictEqual(u8aXOR(false, a, b, c), aXORbXORc)

    u8aXOR(true, a, b, c)
    assert.deepStrictEqual(a, aXORbXORc)
  })
})
