import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'
import PeerId from 'peer-id'

import { NetworkInteractions } from './network'
import { PacketInteractions } from './packet'
import { Mixer } from '../mixer'

class Interactions<Chain extends HoprCoreConnector> {
  public network: NetworkInteractions
  public packet: PacketInteractions<Chain>

  constructor(node: Hopr<Chain>, mixer: Mixer<Chain>, heartbeat: (remotePeer: PeerId) => void) {
    this.network = new NetworkInteractions(node._libp2p, heartbeat)
    this.packet = new PacketInteractions(node, mixer)
  }
}

export { Interactions }
