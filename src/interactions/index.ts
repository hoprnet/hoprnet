import Hopr from '..'
import { PaymentInteractions } from './payments'
import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'

class Interactions<Chain extends HoprCoreConnectorInstance> {
  public payments: PaymentInteractions<Chain>

  constructor(node: Hopr<Chain>) {
    this.payments = new PaymentInteractions<Chain>(node)
  }
}

export { Interactions }
