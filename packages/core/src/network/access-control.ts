import type { PeerId } from '@libp2p/interface-peer-id'
import type NetworkPeers from './network-peers.js'
import { debug } from '@hoprnet/hopr-utils'
import { NetworkPeersOrigin } from './network-peers.js'

// const log = debug('hopr-core:access-control')
const logError = debug('hopr-core:access-control:error')

/**
 * Encapsulates logic to control access behaviours.
 */
export default class AccessControl {
  constructor(
    private networkPeers: NetworkPeers,
    private isAllowedAccessToNetwork: (peerId: PeerId) => Promise<boolean>,
    private closeConnectionsTo: (peerId: PeerId) => void
  ) {}

  private allowConnectionWithPeer(peerId: PeerId, origin: NetworkPeersOrigin): void {
    this.networkPeers.removePeerFromDenied(peerId)
    this.networkPeers.register(peerId, origin)
  }

  private async denyConnectionWithPeer(peerId: PeerId, origin: NetworkPeersOrigin): Promise<void> {
    this.closeConnectionsTo(peerId)
    this.networkPeers.addPeerToDenied(peerId, origin)
  }

  /**
   * Update connection status of a peer.
   * @param peerId the peer's peer id
   * @param origin of the connection
   * @returns true if peer is allowed access
   */
  public async reviewConnection(peerId: PeerId, origin: NetworkPeersOrigin): Promise<boolean> {
    let allowed: boolean = false

    try {
      allowed = await this.isAllowedAccessToNetwork(peerId)
      if (allowed) {
        this.allowConnectionWithPeer(peerId, origin)
      } else {
        this.denyConnectionWithPeer(peerId, origin)
      }
    } catch (error) {
      logError(`unexpected error when reviewing connection ${peerId.toString()} from ${origin}`, error)
    }

    return allowed
  }

  /**
   * Iterate all peers and update their connection status.
   */
  public async reviewConnections(): Promise<void> {
    // Use iterator to prevent from cloning elements
    for (const { id, origin } of this.networkPeers.getAllEntries()) {
      await this.reviewConnection(id, origin)
    }

    // Use iterator to prevent from cloning elements
    for (const { id, origin } of this.networkPeers.getAllDenied()) {
      await this.reviewConnection(id, origin)
    }
  }
}
