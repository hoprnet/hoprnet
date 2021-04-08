import type Hopr from '..'
import PeerId from 'peer-id'

import { PacketInteractions } from './packet'
import { Heartbeat } from './network/heartbeat'

class Interactions {
  heartbeat: Heartbeat
  public packet: PacketInteractions

  constructor(node: Hopr, heartbeat: (remotePeer: PeerId) => void) {
    this.heartbeat = new Heartbeat(node._libp2p, heartbeat)
    this.packet = new PacketInteractions(node)
  }
}

export { Interactions }
