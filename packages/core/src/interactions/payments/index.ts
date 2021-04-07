import Hopr from '../..'

import { OnChainKey } from './onChainKey'

class PaymentInteractions {
  onChainKey: OnChainKey

  constructor(node: Hopr) {
    this.onChainKey = new OnChainKey(node)
  }
}

export { PaymentInteractions }
