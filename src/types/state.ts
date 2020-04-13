import type { Types } from "@hoprnet/hopr-core-connector-interface"
import { u8aConcat } from "@hoprnet/hopr-utils"
import { TicketEpoch, Hash, Public } from '.'
import { Uint8ArrayE } from '../types/extended'

class State extends Uint8ArrayE implements Types.State {
  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      secret: Hash
      pubkey: Public
      epoch: TicketEpoch
    }
  ) {
    if (arr != null && struct == null) {
      super(arr.bytes, arr.offset, State.SIZE)
    } else if (arr == null && struct != null) {
      super(u8aConcat(struct.secret, struct.pubkey, struct.epoch.toU8a()))
    } else {
      throw Error(`Invalid constructor arguments.`)
    }
  }

  get secret() {
    return new Hash(this.subarray(0, Hash.SIZE))
  }

  get pubkey() {
    return new Public(this.subarray(Hash.SIZE, Hash.SIZE + Public.SIZE))
  }

  get epoch() {
    return new TicketEpoch(this.subarray(Hash.SIZE + Public.SIZE, Hash.SIZE + Public.SIZE + TicketEpoch.SIZE))
  }

  static get SIZE() {
    return Hash.SIZE + Public.SIZE + TicketEpoch.SIZE
  }
}

export default State
