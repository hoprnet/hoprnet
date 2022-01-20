import assert from 'assert'

import { u8aEquals } from './equals'

describe('test u8a equals', function () {
  it('array equality', function () {
    const LENGTHS = [15, 8, 7, 4, 3, 2, 1, 0]

    for (const length of LENGTHS) {
      assert(
        u8aEquals(
          new Uint8Array(length).fill(0xff),
          new Uint8Array(length).fill(0xff),
          new Uint8Array(length).fill(0xff)
        ) == true
      )

      if (length > 0) {
        assert(
          u8aEquals(
            new Uint8Array(length).fill(0xaa),
            new Uint8Array(length).fill(0xaa),
            new Uint8Array(length).fill(0xff)
          ) == false
        )
      }
    }
  })

  it('array equality - edge cases', function () {
    const tests = [
      {
        result: false,
        args: [undefined, new Uint8Array(), new Uint8Array()]
      },
      {
        result: false,
        args: [new Uint8Array(), undefined, new Uint8Array()]
      },
      {
        result: false,
        args: [new Uint8Array(1), new Uint8Array(), new Uint8Array()]
      },
      {
        result: false,
        args: [new Uint8Array(), new Uint8Array(1), new Uint8Array()]
      }
    ]

    for (const test of tests) {
      assert(u8aEquals(...test.args) == test.result)
    }
  })
})
