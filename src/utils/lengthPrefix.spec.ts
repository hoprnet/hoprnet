import { encodeWithLengthPrefix, decodeWithLengthPrefix } from './lengthPrefix'
import { u8aEquals } from '@hoprnet/hopr-utils'

import assert from 'assert'

describe('test length prefix', function () {
  it('encode and decode single message', function () {
    const firstMessage = new TextEncoder().encode('first message')

    const decoded = decodeWithLengthPrefix(encodeWithLengthPrefix(firstMessage))

    assert(decoded.length == 1 && u8aEquals(decoded[0], firstMessage))
  })

  it('encode and decode multiple messages', function () {
    const firstMessage = new TextEncoder().encode('first message')
    const secondMessage = new TextEncoder().encode('first message')

    const multiEncoded = Uint8Array.from([
      ...encodeWithLengthPrefix(firstMessage),
      ...encodeWithLengthPrefix(secondMessage)
    ])

    const decoded = decodeWithLengthPrefix(multiEncoded)

    assert(decoded.length == 2 && u8aEquals(decoded[0], firstMessage) && u8aEquals(decoded[1], secondMessage))
  })

  it('encode / decode edge cases', function () {
    assert.throws(
      () => decodeWithLengthPrefix(new Uint8Array()),
      Error(`Unable to read message because given array (size ${0} elements) is too small to include length prefix.`)
    )

    assert.throws(
      () => decodeWithLengthPrefix(Uint8Array.from([0xff, 0xff, 0xff, 0xff])),
      Error(`Invalid length prefix encoding. Encoded length does not fit to given array`)
    )

    assert.throws(
      () => decodeWithLengthPrefix(Uint8Array.from([0x00, 0x00, 0x00, 0x01, 0xff, 0x00, 0x00, 0x00, 0x02, 0x01])),
      Error(`Invalid length prefix encoding. Encoded length does not fit to given array`)
    )
  })
})
