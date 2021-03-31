import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import { isBootstrapNode } from './isBootstrapNode'

/**
 * Get node's peers.
 * @returns an array of peer ids
 */
export function getPeers(
  node: Hopr<HoprCoreConnector>,
  ops: {
    noBootstrapNodes: boolean
  } = {
    noBootstrapNodes: false
  }
): PeerId[] {
  let peers = node.getConnectedPeers()

  if (ops.noBootstrapNodes) {
    peers = peers.filter((peerId) => !isBootstrapNode(node, peerId))
  }

  return peers
}
