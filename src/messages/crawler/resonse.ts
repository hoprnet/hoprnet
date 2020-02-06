import { Status } from '.'

import PeerInfo from 'peer-info'

import { encode, decode } from 'rlp'

import { u8aToNumber, u8aConcat, toU8a, serializePeerInfo, deserializePeerInfo } from '../../utils'

const ENUM_LENGTH = 1

class CrawlResponse extends Uint8Array {
  constructor(
    arr?: Uint8Array,
    struct?: {
      status: Status
      peerInfos?: PeerInfo[]
    }
  ) {
    if (arr != null && struct == null) {
      super(arr)
    } else if (arr == null && struct != null) {
      if (struct.peerInfos != null) {
        super(u8aConcat(toU8a(struct.status, ENUM_LENGTH), encode(struct.peerInfos.map((peerInfo: PeerInfo) => serializePeerInfo(peerInfo)))))
      } else {
        super(u8aConcat(toU8a(struct.status, ENUM_LENGTH)))
      }
    }
  }
  subarray(begin?: number, end?: number): Uint8Array {
    return new Uint8Array(this.buffer, begin, end != null ? end - begin : undefined)
  }

  get statusRaw(): Uint8Array {
    return this.subarray(0, ENUM_LENGTH)
  }

  get status(): Status {
    return u8aToNumber(this.statusRaw)
  }

  get peerInfosRaw(): Uint8Array {
    return this.subarray(ENUM_LENGTH, this.length)
  }

  get peerInfos(): Promise<PeerInfo[]> {
    return Promise.all((decode(this.peerInfosRaw) as Buffer[]).map((arr: Buffer) => deserializePeerInfo(arr)))
  }
}

export { CrawlResponse }
