import { webcrypto } from 'node:crypto'
import { PeerId } from '@libp2p/interface-peer-id'
// @ts-ignore necessary for getrandom crate
globalThis.crypto = webcrypto

import { existsSync, unlinkSync } from 'node:fs'
import assert from 'node:assert'

import { IdentityOptions, HoprKeys, hoprd_keypair_set_panic_hook } from '../lib/hoprd_keypair.js'
import { stringToU8a } from '@hoprnet/hopr-utils'
hoprd_keypair_set_panic_hook()

// Test whether Rust-based Identity write / read functionality works
describe('test HoprKeys', function () {
  const DUMMY_PATH = new URL('./hopr-test-identity', import.meta.url).pathname
  const DUMMY_PASSWORD = 'hopr-unit-test-password'

  const ALICE =
    '0x2ff826bc50030fd5d04debfb8e75cd36204b80b0d875b38f54b8b38978ae597901809772188b60a557e8259513a613d0531f813a967a496fb45e3d8ace28ed5d'

  const ALICE_CHAIN_KEY = '16Uiu2HAm1szVsHgiAiUELinCf9V7A68HAF7E9FCpEzLQvhyyu6FQ'
  const ALICE_PACKET_KEY = '12D3KooWM57EqgWq1xr9B3aYeeXu41uGGpSAQmF6qUqPgi27T1Jv'

  it('export peerIds', async function () {
    let keys = new HoprKeys()

    assert(((await keys.chainKeyPeerId) as PeerId).type === 'secp256k1')
    assert(((await keys.packetKeyPeerId) as PeerId).type === 'Ed25519')
  })

  it('e2e test: initialize, write, read', async function () {
    let keys = HoprKeys.init(new IdentityOptions(false, DUMMY_PATH, DUMMY_PASSWORD, true, stringToU8a(ALICE)))

    const chainPeerId = (await keys.chainKeyPeerId) as PeerId
    const packetPeerId = (await keys.packetKeyPeerId) as PeerId

    assert(chainPeerId.type === 'secp256k1')
    assert(packetPeerId.type === 'Ed25519')

    assert(chainPeerId.toString() === ALICE_CHAIN_KEY)
    assert(packetPeerId.toString() === ALICE_PACKET_KEY)

    assert(existsSync(DUMMY_PATH))

    let keysCopy = HoprKeys.read_eth_keystore(DUMMY_PATH, DUMMY_PASSWORD)

    const chainPeerIdCopy = (await keysCopy.chainKeyPeerId) as PeerId
    const packetPeerIdCopy = (await keysCopy.packetKeyPeerId) as PeerId

    assert(chainPeerIdCopy.type === 'secp256k1')
    assert(packetPeerIdCopy.type === 'Ed25519')

    assert(chainPeerIdCopy.toString() === ALICE_CHAIN_KEY)
    assert(packetPeerIdCopy.toString() === ALICE_PACKET_KEY)

    unlinkSync(DUMMY_PATH)
  })

  it('e2e test: initialize, write, read - without 0x prefix', async function () {
    let keys = HoprKeys.init(new IdentityOptions(false, DUMMY_PATH, DUMMY_PASSWORD, true, stringToU8a(ALICE.slice(2))))

    const chainPeerId = (await keys.chainKeyPeerId) as PeerId
    const packetPeerId = (await keys.packetKeyPeerId) as PeerId

    assert(chainPeerId.type === 'secp256k1')
    assert(packetPeerId.type === 'Ed25519')

    assert(chainPeerId.toString() === ALICE_CHAIN_KEY)
    assert(packetPeerId.toString() === ALICE_PACKET_KEY)

    assert(existsSync(DUMMY_PATH))

    let keysCopy = HoprKeys.read_eth_keystore(DUMMY_PATH, DUMMY_PASSWORD)

    const chainPeerIdCopy = (await keysCopy.chainKeyPeerId) as PeerId
    const packetPeerIdCopy = (await keysCopy.packetKeyPeerId) as PeerId

    assert(chainPeerIdCopy.type === 'secp256k1')
    assert(packetPeerIdCopy.type === 'Ed25519')

    assert(chainPeerIdCopy.toString() === ALICE_CHAIN_KEY)
    assert(packetPeerIdCopy.toString() === ALICE_PACKET_KEY)

    unlinkSync(DUMMY_PATH)
  })

  it('fail loading non-existing key', function () {
    assert.throws(
      () => HoprKeys.init(new IdentityOptions(false, DUMMY_PATH, DUMMY_PASSWORD, true)),
      "Error: Key store file does not exist or could not decrypt it. Maybe using the wrong '--password'? Otherwise try again with '--initialize' to overwrite the existing key store. THIS WILL DESTROY THE PREVIOUS KEY"
    )
  })
})
