import PeerId from 'peer-id'

import { u8aConcat } from '@hoprnet/hopr-utils'
import { pubKeyToPeerId } from '../../utils'

const PUBLIC_KEY_LENGTH = 33

class ForwardPacket extends Uint8Array {
  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      destination: PeerId
      sender: PeerId
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
      super(
        u8aConcat(
          struct.destination.pubKey.marshal(),
          struct.sender.pubKey.marshal(),
          struct.payload || new Uint8Array()
        )
      )
    }
  }

  subarray(begin: number = 0, end?: number): Uint8Array {
    return new Uint8Array(
      this.buffer,
      begin + this.byteOffset,
      end != null ? end - begin : undefined
    )
  }

  get destination(): Uint8Array {
    return this.subarray(0, PUBLIC_KEY_LENGTH)
  }

  get destinationPeerId(): Promise<PeerId> {
    return pubKeyToPeerId(this.destination)
  }

  get sender(): Uint8Array {
    return this.subarray(PUBLIC_KEY_LENGTH, PUBLIC_KEY_LENGTH + PUBLIC_KEY_LENGTH)
  }

  get senderPeerId(): Promise<PeerId> {
    return pubKeyToPeerId(this.sender)
  }

  get payload(): Uint8Array {
    return this.subarray(PUBLIC_KEY_LENGTH + PUBLIC_KEY_LENGTH)
  }
}

export { ForwardPacket }
