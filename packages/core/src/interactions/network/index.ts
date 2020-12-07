import { Heartbeat } from './heartbeat'
import { LibP2P } from '../../'
import PeerId from 'peer-id'

class NetworkInteractions {
  heartbeat: Heartbeat

  constructor(node: LibP2P, heartbeat: (remotePeer: PeerId) => void) {
    this.heartbeat = new Heartbeat(node, heartbeat)
  }
}

export { NetworkInteractions }
