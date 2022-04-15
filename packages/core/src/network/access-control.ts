import type PeerId from 'peer-id'
import type NetworkPeers from './network-peers'

/**
 * Encapsulates logic to control access behaviours.
 */
export default class AccessControl {
  constructor(
    private networkPeers: NetworkPeers,
    private isAllowedAccess: (peerId: PeerId) => Promise<boolean>,
    private closeConnectionsTo: (peerId: PeerId) => Promise<void>
  ) {}

  private async allowConnectionWithPeer(peerId: PeerId, origin: string): Promise<void> {
    this.networkPeers.removePeerFromDenied(peerId)
    this.networkPeers.register(peerId, origin)
  }

  private async denyConnectionWithPeer(peerId: PeerId, origin: string): Promise<void> {
    this.networkPeers.addPeerToDenied(peerId, origin)
    await this.closeConnectionsTo(peerId)
  }

  /**
   * Update connection status of a peer.
   * @param peerId the peer's peer id
   * @param origin of the connection
   * @returns true if peer is allowed access
   */
  public async reviewConnection(peerId: PeerId, origin: string): Promise<boolean> {
    const allowed = this.isAllowedAccess(peerId)
    if (allowed) await this.allowConnectionWithPeer(peerId, origin)
    else await this.denyConnectionWithPeer(peerId, origin)
    return allowed
  }

  /**
   * Iterate all peers and update their connection status.
   */
  public async reviewConnections(): Promise<void> {
    const allPeers = [...this.networkPeers.allEntries(), ...this.networkPeers.getAllDenied()]
    for (const { id, origin } of allPeers) await this.reviewConnection(id, origin)
  }
}
