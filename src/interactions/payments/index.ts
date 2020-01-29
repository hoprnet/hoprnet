import Hopr from '../..'
import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'

import { Opening } from './open'
import { OnChainKey } from './onChainKey'

class PaymentInteractions<Chain extends HoprCoreConnectorInstance> {
  open: Opening<Chain>
  onChainKey: OnChainKey<Chain>

  constructor(node: Hopr<Chain>) {
    this.open = new Opening(node)
    this.onChainKey = new OnChainKey(node)
  }
}

export { PaymentInteractions }
