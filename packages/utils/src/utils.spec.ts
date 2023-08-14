import { expandVars } from './utils.js'
import assert from 'assert'
import { peerIdFromKeys } from '@libp2p/peer-id'
import { OffchainKeypair } from './types.js'

describe('utils', () => {
  it('expands vars', async () => {
    assert(expandVars('simple string', {}) === 'simple string')
    assert(expandVars('simple ${foo}', { foo: 'bar' }) === 'simple bar')
    assert.throws(() => expandVars('simple string ${foo}', { not_bar: 1 }))
  })

  it('keypair to peerid', async () => {
    let kp = OffchainKeypair.random()
    let peerId = await peerIdFromKeys(kp.public().serialize(), kp.secret())

    assert.equal(peerId.type, 'Ed25519', 'must yield ed25519 peerid')
    assert.equal(peerId.toString(), kp.to_peerid_str(), 'must yield same peer ids')
    assert(peerId.privateKey, 'must have private key')

    assert.equal(peerId.publicKey, kp.public(), 'public keys must be equal')
    assert.equal(peerId.privateKey, kp.secret(), 'private keys must be equal')
  })
})
