import { CID } from 'multiformats/cid'
import * as raw from 'multiformats/codecs/raw'
import { sha256 } from 'multiformats/hashes/sha2'
import type PeerId from 'peer-id'

export async function createRelayerKey(relayer: PeerId): Promise<CID> {
  console.log(new TextEncoder().encode(`/relay/${relayer.toB58String()}`))

  const bytes = raw.encode(new TextEncoder().encode(`/relay/${relayer.toB58String()}`))
  const hash = await sha256.digest(bytes)

  return CID.create(1, raw.code, hash)
}
