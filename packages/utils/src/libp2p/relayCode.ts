// @ts-ignore untyped dedpendency
import { CID } from 'multiformats/cid'
// @ts-ignore untyped dedpendency
import { code, encode } from 'multiformats/codecs/raw'
// @ts-ignore untyped dedpendency
import { sha256 } from 'multiformats/hashes/sha2'
import type { PeerId } from '@libp2p/interface-peer-id'

/**
 * Creates a DHT entry to give relays the opportunity to signal
 * other nodes in the network that they act as a relay for the given
 * node.
 * @param destination peerId of the node for which relay services are provided
 * @returns the DHT entry key
 */
export function createRelayerKey(destination: PeerId): CID {
  const bytes = encode(new TextEncoder().encode(`/relay/${destination.toString()}`))
  const hash = sha256.digest(bytes)

  return CID.create(1, code, hash)
}
