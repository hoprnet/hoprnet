import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'
import PeerId from 'peer-id'

import { PaymentInteractions } from './payments'
import { PacketInteractions } from './packet'
import { Heartbeat } from './network/heartbeat'

class Interactions<Chain extends HoprCoreConnector> {
  public payments: PaymentInteractions<Chain>
  heartbeat: Heartbeat
  public packet: PacketInteractions<Chain>

  constructor(node: Hopr<Chain>, heartbeat: (remotePeer: PeerId) => void) {
    this.payments = new PaymentInteractions(node)
    this.heartbeat = new Heartbeat(node._libp2p, heartbeat)
    this.packet = new PacketInteractions(node)
  }
}

export { Interactions }
