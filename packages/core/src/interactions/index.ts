import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'
import type Multiaddr from 'multiaddr'
import PeerId from 'peer-id'

import { PaymentInteractions } from './payments'
import { NetworkInteractions } from './network'
import { PacketInteractions } from './packet'
import { Mixer } from '../mixer'

class Interactions<Chain extends HoprCoreConnector> {
  public payments: PaymentInteractions<Chain>
  public network: NetworkInteractions
  public packet: PacketInteractions<Chain>

  constructor(
    node: Hopr<Chain>,
    mixer: Mixer<Chain>,
    answerCrawl: (addr: Multiaddr) => Promise<Multiaddr[]>,
    heartbeat: (remotePeer: PeerId) => void
  ) {
    this.payments = new PaymentInteractions(node)
    this.network = new NetworkInteractions(node._libp2p, answerCrawl, heartbeat)
    this.packet = new PacketInteractions(node, mixer)
  }
}

export { Interactions }
