import { privKeyToPeerId } from './privKeyToPeerId'
import PeerId from 'peer-id'
import assert from 'assert'

describe(`converting privKey to peerId`, function () {
  it('convert back and forth', async function () {
    const pId = await PeerId.create({ keyType: 'secp256k1' })

    const deserialized = privKeyToPeerId(pId.privKey.marshal())

    assert(deserialized.equals(pId))
  })
})
