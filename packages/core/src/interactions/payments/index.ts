import Hopr from '../..'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

import { OnChainKey } from './onChainKey'

class PaymentInteractions<Chain extends HoprCoreConnector> {
  onChainKey: OnChainKey<Chain>

  constructor(node: Hopr<Chain>) {
    this.onChainKey = new OnChainKey(node)
  }
}

export { PaymentInteractions }
