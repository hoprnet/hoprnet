import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'
import PeerId from 'peer-id'
import type { Connection } from '../@types/transport'

import { PaymentInteractions } from './payments'
import { NetworkInteractions } from './network'
import { PacketInteractions } from './packet'

class Interactions<Chain extends HoprCoreConnector> {
  public payments: PaymentInteractions<Chain>
  public network: NetworkInteractions
  public packet: PacketInteractions<Chain>

  constructor(
    node: Hopr<Chain>, 
    handleCrawlRequest: (conn: Connection) => void,
    heartbeat: (remotePeer: PeerId) => void,
  ) {
    this.payments = new PaymentInteractions(node)
    this.network = new NetworkInteractions(node, handleCrawlRequest, heartbeat)
    this.packet = new PacketInteractions(node)
  }
}

export { Interactions }
