import { pubKeyToPeerId } from './pubKeyToPeerId'
import PeerId from 'peer-id'
import assert from 'assert'

describe(`converting pubKey to peerId`, function () {
  it('convert back and forth', async function () {
    const pId = await PeerId.create({ keyType: 'secp256k1' })

    const deserialized = pubKeyToPeerId(pId.pubKey.marshal())

    assert(deserialized.equals(pId))
  })
})
