import { CID } from 'multiformats/cid'
import * as raw from 'multiformats/codecs/raw'
import { sha256 } from 'multiformats/hashes/sha2'
import type PeerId from 'peer-id'

/**
 * Creates a DHT entry to give relays the opportunity to signal
 * other nodes in the network that they act as a relay for the given
 * node.
 * @param destination peerId of the node for which relay services are provided
 * @returns the DHT entry key
 */
export async function createRelayerKey(destination: PeerId): Promise<CID> {
  const bytes = raw.encode(new TextEncoder().encode(`/relay/${destination.toB58String()}`))
  const hash = await sha256.digest(bytes)

  return CID.create(1, raw.code, hash)
}
