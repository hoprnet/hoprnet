import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type { GlobalState } from '../commands/abstractCommand'
import PeerId from 'peer-id'
// @ts-ignore
import Multihash from 'multihashes'
import bs58 from 'bs58'
import { addPubKey } from '@hoprnet/hopr-core/lib/utils'
import { getPeersIdsAsString } from './openChannels'

/**
 * Takes a string, and checks whether it's an alias or a valid peerId,
 * then it generates a PeerId instance and returns it.
 *
 * @param peerIdString query that contains the peerId
 * @returns a 'PeerId' instance
 */
export async function checkPeerIdInput(peerIdString: string, state?: GlobalState): Promise<PeerId> {
  try {
    if (typeof state !== 'undefined' && state.aliases.has(peerIdString)) {
      return state.aliases.get(peerIdString)!
    }

    // Throws an error if the Id is invalid
    Multihash.decode(bs58.decode(peerIdString))

    return await addPubKey(PeerId.createFromB58String(peerIdString))
  } catch (err) {
    throw Error(`Invalid peerId. ${err.message}`)
  }
}

export function getPeerIdsAndAliases(node: Hopr<HoprCoreConnector>, state: GlobalState): string[] {
  return getPeersIdsAsString(node, {
    noBootstrapNodes: true,
  }).concat(Array.from(state.aliases.keys()))
}
