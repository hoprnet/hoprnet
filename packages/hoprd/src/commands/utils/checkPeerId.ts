import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type { GlobalState } from '../abstractCommand'
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
    if (typeof state !== 'undefined' && state.aliases && state.aliases.has(peerIdString)) {
      return state.aliases.get(peerIdString)!
    }

    // Throws an error if the Id is invalid
    Multihash.decode(bs58.decode(peerIdString))

    return await addPubKey(PeerId.createFromB58String(peerIdString))
  } catch (err) {
    throw Error(`Invalid peerId. ${err.message}`)
  }
}

/**
 * Returns a list of peerIds and aliases.
 * Optionally, you may choose various options.
 *
 * @param node hopr node
 * @param state global state
 * @param ops.noBootstrapNodes do not return any bootstrap nodes
 * @param ops.returnAlias when available, return the peerIds's alias
 * @param ops.mustBeOnline only return online peerIds
 * @returns an array of peerIds / aliases
 */
export function getPeerIdsAndAliases(
  node: Hopr<HoprCoreConnector>,
  state: GlobalState,
  ops: {
    noBootstrapNodes: boolean
    returnAlias: boolean
    mustBeOnline: boolean
  } = {
    noBootstrapNodes: false,
    returnAlias: false,
    mustBeOnline: false
  }
): string[] {
  let peerIds = new Map<
    string,
    {
      value: string
      isOnline: boolean
      alias?: string
    }
  >()

  // add online peer ids into map
  getPeersIdsAsString(node, {
    noBootstrapNodes: ops.noBootstrapNodes
  }).forEach((value) => {
    peerIds.set(value, {
      value,
      isOnline: true
    })
  })

  // add aliases peer ids into map
  Array.from(state.aliases.entries()).forEach(([alias, peerId]) => {
    const value = peerId.toB58String()

    peerIds.set(value, {
      value,
      isOnline: peerIds.has(value),
      alias
    })
  })

  // remove offline nodes
  if (ops.mustBeOnline) {
    for (const item of peerIds.values()) {
      if (item.isOnline) continue
      peerIds.delete(item.value)
    }
  }

  // return alias if it's available
  if (ops.returnAlias) {
    return Array.from(peerIds.values()).map((item) => item.alias || item.value)
  }

  // return value
  return Array.from(peerIds.values()).map((item) => item.value)
}
