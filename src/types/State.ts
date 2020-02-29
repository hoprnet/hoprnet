import TypeConstructors from '@hoprnet/hopr-core-connector-interface/src/types'
import { typedClass } from 'src/tsc/utils'
import { TicketEpoch, Hash, Public } from '.'
import { Uint8Array } from 'src/types/extended'

@typedClass<TypeConstructors['State']>()
class State extends Uint8Array {
  secret: Hash
  pubkey: Public
  epoch: TicketEpoch

  static get SIZE() {
    return Hash.SIZE + Public.SIZE + TicketEpoch.SIZE
  }
}

export default State
