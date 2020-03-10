import TypeConstructors from '@hoprnet/hopr-core-connector-interface/src/types'
import { typedClass } from '../tsc/utils'
import { TicketEpoch, Hash, Public } from '.'
import { Uint8ArrayE } from '../types/extended'

@typedClass<TypeConstructors['State']>()
class State extends Uint8ArrayE {
  secret: Hash
  pubkey: Public
  epoch: TicketEpoch

  static get SIZE() {
    return Hash.SIZE + Public.SIZE + TicketEpoch.SIZE
  }
}

export default State
