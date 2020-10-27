import assert from 'assert'
import PeerId from 'peer-id'
import { convertPubKeyFromPeerId, convertPubKeyFromB58String } from '.'
// @ts-ignore
import * as multihashes from 'multihashes'

describe(`test convertPubKeyFromPeerId`, function () {
  it(`should equal to a newly created pubkey from PeerId`, async function () {
    const id = await PeerId.create({ keyType: 'secp256k1', bits: 256 })
    const pubKey = await convertPubKeyFromPeerId(id)
    assert(id.pubKey.toString() === pubKey.toString())
  })
  it(`should equal to pubkey from a PeerId CID`, async function () {
    const testIdB58String = '16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg'
    const pubKey = await convertPubKeyFromB58String(testIdB58String)
    const id = PeerId.createFromCID(testIdB58String)
    assert(id.pubKey.toString() === pubKey.toString())
  })
})
