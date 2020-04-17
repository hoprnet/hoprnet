import assert from 'assert'
import { randomBytes } from 'crypto'
import PeerId from 'peer-id'
import { ForwardPacket } from '.'

import { u8aEquals } from '@hoprnet/hopr-utils'

describe('test forward packet', function () {
  it('should create a forward packet', async function () {
    const destination = await PeerId.create({ keyType: 'secp256k1' })
    const payload = randomBytes(41)

    const packet = new ForwardPacket(undefined, {
      destination,
      payload,
    })

    const serializedPacket = new ForwardPacket({
      bytes: packet.buffer,
      offset: packet.byteOffset,
    })

    assert((await PeerId.createFromPubKey(Buffer.from(serializedPacket.destination))).isEqual(destination), `Recovered peerId must be the same as before.`)

    assert(u8aEquals(payload, serializedPacket.payload))
  })
})
