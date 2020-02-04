import Hopr from '..'
import { PaymentInteractions } from './payments'
import { NetworkInteractions } from './network'
import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'

export { Duplex, Sink, Source } from './abstractInteraction'

class Interactions<Chain extends HoprCoreConnectorInstance> {
  public payments: PaymentInteractions<Chain>
  public network: NetworkInteractions<Chain>

  constructor(node: Hopr<Chain>) {
    this.payments = new PaymentInteractions<Chain>(node)
    this.network = new NetworkInteractions<Chain>(node)
  }
}

export { Interactions }
