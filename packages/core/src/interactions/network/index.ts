/// <reference path="../../@types/libp2p.ts" />

import { Heartbeat } from './heartbeat'
import LibP2P, { Handler } from 'libp2p'
import PeerId from 'peer-id'

class NetworkInteractions {
  heartbeat: Heartbeat

  constructor(
    node: LibP2P,
    heartbeat: (remotePeer: PeerId) => void,
    dialProtocol: (counterparty: PeerId, protocols: string[], seconds: number) => Promise<Handler | void>
  ) {
    this.heartbeat = new Heartbeat(node, heartbeat, dialProtocol)
  }
}

export { NetworkInteractions }
