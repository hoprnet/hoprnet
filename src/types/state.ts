import type { Types } from "@hoprnet/hopr-core-connector-interface"
import { TicketEpoch, Hash, Public } from '.'
import { Uint8ArrayE } from '../types/extended'

class State extends Uint8ArrayE implements Types.State {
  secret: Hash
  pubkey: Public
  epoch: TicketEpoch

  static get SIZE() {
    return Hash.SIZE + Public.SIZE + TicketEpoch.SIZE
  }
}

export default State
