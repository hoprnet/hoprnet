import type Hopr from '@hoprnet/hopr-core'
import type { State } from '../../types.js'
import type { PeerId } from '@libp2p/interface-peer-id'
import { peerIdFromString } from '@libp2p/peer-id'

/**
 * Takes a string, and checks whether it's an alias or a valid peerId,
 * then it generates a PeerId instance and returns it.
 *
 * @param peerIdString query that contains the peerId
 * @returns a 'PeerId' instance
 */
export function checkPeerIdInput(peerIdString: string, state?: State): PeerId {
  try {
    if (typeof state !== 'undefined' && state.aliases && state.aliases.has(peerIdString)) {
      return state.aliases.get(peerIdString)!
    }

    return peerIdFromString(peerIdString)
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
 * @param ops.returnAlias when available, return the peerIds's alias
 * @param ops.mustBeOnline only return online peerIds
 * @returns an array of peerIds / aliases
 */
export function getPeerIdsAndAliases(
  node: Hopr,
  state: State,
  ops: {
    returnAlias: boolean
    mustBeOnline: boolean
  } = {
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
  let peers = node.getConnectedPeers()

  // update map
  peers.forEach((value) => {
    peerIds.set(value.toString(), {
      value: value.toString(),
      isOnline: true
    })
  })

  // add aliases peer ids into map
  Array.from(state.aliases.entries()).forEach(([alias, peerId]) => {
    const value = peerId.toString()

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
