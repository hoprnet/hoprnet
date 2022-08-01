import type { PeerId } from '@libp2p/interface-peer-id'
import { peerIdFromPeerId } from '@libp2p/peer-id'
import { keys } from '@libp2p/crypto'

import { stringToU8a } from '../u8a/index.js'
// @ts-ignore untyped dependency
import { identity } from 'multiformats/hashes/identity'

import secp256k1 from 'secp256k1'

const PRIVKEY_LENGTH = 32

/**
 * Converts a plain compressed ECDSA private key over the curve `secp256k1`
 * to a peerId in order to use it with libp2p.
 * It equips the generated peerId with private key and public key.
 *
 * @param privKey the plain private key
 */
export function privKeyToPeerId(privKey: Uint8Array | string): PeerId {
  let internalPrivKey: Uint8Array
  switch (typeof privKey) {
    case 'string':
      const matched = privKey.match(/(?<=^0x|^)[0-9a-fA-F]{64}/)

      if (!matched) {
        throw Error(`Invalid input argument. Either key length or key characters were incorrect.`)
      }
      internalPrivKey = stringToU8a(privKey, PRIVKEY_LENGTH)
      break
    case 'object':
      if (privKey.length != PRIVKEY_LENGTH) {
        throw Error(
          `Invalid private key. Expected a buffer of size ${PRIVKEY_LENGTH} bytes. Got one of ${privKey.length} bytes.`
        )
      }
      internalPrivKey = privKey
      break
    default:
      throw Error(`Invalid input arguments`)
  }

  const secp256k1PrivKey = new keys.supportedKeys.secp256k1.Secp256k1PrivateKey(
    internalPrivKey,
    secp256k1.publicKeyCreate(internalPrivKey)
  )

  return peerIdFromPeerId({
    type: 'secp256k1',
    multihash: identity.digest(secp256k1PrivKey.public.bytes),
    privateKey: secp256k1PrivKey.bytes,
    publicKey: secp256k1PrivKey.public.bytes
  })
}
