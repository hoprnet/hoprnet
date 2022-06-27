import type { PeerId } from '@libp2p/interface-peer-id'
import { peerIdFromPeerId } from '@libp2p/peer-id'
import { keys } from '@libp2p/crypto'
import { stringToU8a } from '../u8a/index.js'
import { identity } from 'multiformats/hashes/identity'

const COMPRESSED_PUBLIC_KEY_LENGTH = 33

/**
 * Converts a plain compressed ECDSA public key over the curve `secp256k1`
 * to a peerId in order to use it with libp2p.
 *
 * @notice Libp2p stores the keys in format that is derived from `protobuf`.
 * Using `libsecp256k1` directly does not work.
 *
 * @param pubKey the plain public key
 */
export function pubKeyToPeerId(pubKey: Uint8Array | string): PeerId {
  let internalPubKey: Uint8Array
  switch (typeof pubKey) {
    case 'string':
      const matched = pubKey.match(/(?<=^0x|^)[0-9a-fA-F]{66}/)

      if (!matched) {
        throw Error(`Invalid input argument. Either key length or key characters were incorrect.`)
      }
      pubKey = stringToU8a(pubKey, COMPRESSED_PUBLIC_KEY_LENGTH)
      break
    case 'object':
      if (pubKey.length != COMPRESSED_PUBLIC_KEY_LENGTH) {
        throw Error(
          `Invalid public key. Expected a buffer of size ${COMPRESSED_PUBLIC_KEY_LENGTH} bytes. Got one of ${pubKey.length} bytes.`
        )
      }
      internalPubKey = pubKey
      break
    default:
      throw Error(`Invalid input arguments`)
  }

  const secp256k1PubKey = new keys.supportedKeys.secp256k1.Secp256k1PublicKey(pubKey)

  return peerIdFromPeerId({
    type: 'secp256k1',
    multihash: identity.digest(secp256k1PubKey.bytes),
    publicKey: secp256k1PubKey.bytes
  })
}
