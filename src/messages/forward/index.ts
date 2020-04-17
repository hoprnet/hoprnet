import PeerId from 'peer-id'

import { u8aConcat } from '@hoprnet/hopr-utils'

const PUBLIC_KEY_LENGTH = 37

class ForwardPacket extends Uint8Array {
  constructor(
    arr?: {
      bytes: ArrayBuffer,
      offset: number
    },
    struct?: {
      destination: PeerId
      payload?: Uint8Array
    }
  ) {
    if (arr != null && struct == null) {
      super(arr.bytes, arr.offset)
      try {
        PeerId.createFromBytes(Buffer.from(this.destination))
      } catch {
        throw Error('Invalid peerId.')
      }
    } else if (arr == null && struct != null) {
      super(u8aConcat(struct.destination.marshalPubKey(), struct.payload || new Uint8Array()))
    }
  }

  subarray(begin: number = 0, end?: number): Uint8Array {
    return new Uint8Array(this.buffer, begin, end != null ? end - begin : undefined)
  }

  get destination() {
    return this.subarray(0, PUBLIC_KEY_LENGTH)
  }

  get payload() {
    return this.subarray(PUBLIC_KEY_LENGTH)
  }
}

export { ForwardPacket }
