import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'

/**
 * Checks whether the given PeerId belongs to any known bootstrap node.
 *
 * @param peerId
 */
export function isBootstrapNode(node: Hopr<HoprCoreConnector>, peerId: PeerId): boolean {
  for (let i = 0; i < node.bootstrapServers.length; i++) {
    if (peerId.toB58String() === node.bootstrapServers[i].getPeerId()) {
      return true
    }
  }

  return false
}
