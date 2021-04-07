import type Hopr from '..'
import PeerId from 'peer-id'

import { PaymentInteractions } from './payments'
import { PacketInteractions } from './packet'
import { Heartbeat } from './network/heartbeat'

class Interactions {
  public payments: PaymentInteractions
  heartbeat: Heartbeat
  public packet: PacketInteractions

  constructor(node: Hopr, heartbeat: (remotePeer: PeerId) => void) {
    this.payments = new PaymentInteractions(node)
    this.heartbeat = new Heartbeat(node._libp2p, heartbeat)
    this.packet = new PacketInteractions(node)
  }
}

export { Interactions }
