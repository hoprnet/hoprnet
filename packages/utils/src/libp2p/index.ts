import PeerId from 'peer-id'
import {keys, PublicKey} from 'libp2p-crypto'
// @ts-ignore
import * as multihashes from 'multihashes'

/**
 * Regular expresion used to match b58Strings
 *
 */

export const b58StringRegex = /16Uiu2HA[A-Za-z0-9]{1,45}/i

/**
 * Takes a peerId and returns its corresponding public key.
 *
 * @param peerId the PeerId used to generate a public key
 */
export async function convertPubKeyFromPeerId(peerId: PeerId): Promise<PublicKey> {
  return await keys.unmarshalPublicKey(multihashes.decode(peerId.toBytes()).digest)
}

/**
 *
 * Takes a B58String and converts them to a PublicKey
 *
 * @param string the B58String used to represent the PeerId
 */
export async function convertPubKeyFromB58String(b58string: string): Promise<PublicKey> {
  return await convertPubKeyFromPeerId(PeerId.createFromB58String(b58string))
}

/**
 *
 * Returns true or false if given string does not contain a b58string
 *
 * @param string arbitrary content with maybe a b58string
 */
export function hasB58String(content: string): Boolean {
  const hasMatcheableContent = content.match(b58StringRegex)
  if (hasMatcheableContent) {
    const [maybeB58String] = hasMatcheableContent
    const b58String = maybeB58String.substr(0, 53)
    return b58String.length === 53
  } else {
    return false
  }
}

/**
 *
 * Returns the b58String within a given content. Returns empty string if none is found.
 *
 * @param string arbitrary content with maybe a b58string
 */
export function getB58String(content: string): string {
  const hasMatcheableContent = content.match(b58StringRegex)
  if (hasMatcheableContent) {
    const [maybeB58String] = hasMatcheableContent
    const b58String = maybeB58String.substr(0, 53)
    return b58String
  } else {
    return ''
  }
}
