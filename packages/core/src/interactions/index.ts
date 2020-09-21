import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'

import { PaymentInteractions } from './payments'
import { NetworkInteractions } from './network'
import { PacketInteractions } from './packet'

class Interactions<Chain extends HoprCoreConnector> {
  public payments: PaymentInteractions<Chain>
  public network: NetworkInteractions<Chain>
  public packet: PacketInteractions<Chain>

  constructor(node: Hopr<Chain>) {
    this.payments = new PaymentInteractions(node)
    this.network = new NetworkInteractions(node)
    this.packet = new PacketInteractions(node)
  }
}

export { Interactions }
