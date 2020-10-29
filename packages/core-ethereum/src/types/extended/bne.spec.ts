import assert from 'assert'
import BNE from './bne'

import {u8aEquals} from '@hoprnet/hopr-utils'

describe('test BNE', function () {
  it('should return a Uint8Array', function () {
    const number = 1

    assert(u8aEquals(new BNE(number).toU8a(), new Uint8Array([number])), 'check if BNE u8a array is correct')
  })
})
