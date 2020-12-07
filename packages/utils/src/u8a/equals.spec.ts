import assert from 'assert'

import { u8aEquals } from './equals'

describe('test u8a equals', function () {
  it('should check whether two (or more) Uint8Arrays are equal', function () {
    assert(u8aEquals(new Uint8Array(), new Uint8Array()), `check empty array`)

    assert(
      !u8aEquals(new Uint8Array(1).fill(0xff), new Uint8Array(1).fill(0)),
      `random data should be with high probability not equal`
    )

    assert(
      !u8aEquals(new Uint8Array(1), new Uint8Array(2)),
      `random data should be with high probability not equal, different size`
    )

    assert(u8aEquals(new Uint8Array(32).fill(0xff), new Uint8Array(32).fill(0xff)), `check equal arrays`)

    assert(
      !u8aEquals(new Uint8Array(32).fill(0xff), new Uint8Array(32).fill(0xff), new Uint8Array(32).fill(0xaa)),
      `check different arrays`
    )

    assert(
      !u8aEquals(new Uint8Array(1).fill(1), new Uint8Array(1).fill(2), new Uint8Array(3).fill(3)),
      `random data should be with high probability not equal`
    )

    assert(!u8aEquals(new Uint8Array(), new Uint8Array(), undefined))

    assert(u8aEquals(new Uint8Array(7).fill(0xff), new Uint8Array(7).fill(0xff)))

  })
})
