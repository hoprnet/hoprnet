import { pubKeyToPeerId } from './pubKeyToPeerId.js'
import { createSecp256k1PeerId } from '@libp2p/peer-id-factory'
import assert from 'assert'

describe(`converting pubKey to peerId`, function () {
  it('convert back and forth', async function () {
    const pId = await createSecp256k1PeerId()

    const deserialized = pubKeyToPeerId(pId.pubKey.marshal())

    assert(deserialized.equals(pId))
  })
})
