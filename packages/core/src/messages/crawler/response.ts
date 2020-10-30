import {CrawlStatus} from '.'
import {encode, decode} from 'rlp'
import {u8aConcat, u8aToNumber, toU8a} from '@hoprnet/hopr-utils'
import Multiaddr from 'multiaddr'

const ENUM_LENGTH = 1

class CrawlResponse extends Uint8Array {
  constructor(
    arr?: Uint8Array,
    struct?: {
      status: CrawlStatus
      addresses?: Multiaddr[]
    }
  ) {
    if (arr != null && struct == null) {
      super(arr)
    } else if (arr == null && struct != null) {
      if (struct.addresses == null) {
        if (struct.status == CrawlStatus.OK) {
          throw Error(`Cannot have successful crawling responses without any addresses.`)
        }
        super(u8aConcat(toU8a(struct.status, ENUM_LENGTH)))
      } else if (struct.status == CrawlStatus.OK) {
        super(u8aConcat(toU8a(struct.status, ENUM_LENGTH), encode(struct.addresses.map((ma: Multiaddr) => ma.bytes))))
      } else {
        throw Error(`Invalid creation parameters.`)
      }
    }
  }

  slice(begin: number = 0, end?: number) {
    return this.subarray(begin, end)
  }

  subarray(begin: number = 0, end?: number): Uint8Array {
    return new Uint8Array(this.buffer, begin, end != null ? end - begin : undefined)
  }

  get statusRaw(): Uint8Array {
    return this.subarray(0, ENUM_LENGTH)
  }

  get status(): CrawlStatus {
    return u8aToNumber(this.statusRaw)
  }

  get addressesRaw(): Uint8Array {
    return this.subarray(ENUM_LENGTH, this.length)
  }

  get addresses(): Promise<Multiaddr[]> {
    return Promise.all((decode(this.addressesRaw) as Buffer[]).map((arr: Buffer) => new Multiaddr(arr)))
  }
}

export {CrawlResponse}
