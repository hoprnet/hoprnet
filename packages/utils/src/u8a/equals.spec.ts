import assert from 'assert'

import { u8aEquals } from './equals'

import { randomBytes } from 'crypto'

describe('test u8a equals', function () {
  it('should check whether two (or more) Uint8Arrays are equal', function () {
    assert(u8aEquals(new Uint8Array(), new Uint8Array()), `check empty array`)

    assert(!u8aEquals(randomBytes(32), randomBytes(32)), `random data should be with high probability not equal`)

    assert(
      !u8aEquals(randomBytes(32), randomBytes(31)),
      `random data should be with high probability not equal, different size`
    )

    assert(u8aEquals(new Uint8Array(32).fill(0xff), new Uint8Array(32).fill(0xff)), `check equal arrays`)

    assert(
      !u8aEquals(new Uint8Array(32).fill(0xff), new Uint8Array(32).fill(0xff), new Uint8Array(32).fill(0xaa)),
      `check different arrays`
    )

    assert(
      !u8aEquals(randomBytes(32), randomBytes(32), randomBytes(32)),
      `random data should be with high probability not equal`
    )

    // @ts-ignore
    assert.throws(() => u8aEquals(new Uint8Array(), undefined), `check undefined b`)

    // @ts-ignore
    assert.throws(() => u8aEquals(new Uint8Array(), new Uint8Array(), undefined), `check undefined rest`)
  })
})
