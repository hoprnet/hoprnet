import { CrawlStatus } from '.'
import { u8aToNumber, toU8a, u8aToHex } from '@hoprnet/hopr-utils'
import Multiaddr from 'multiaddr'
import debug from 'debug'

const log = debug('hopr-core:crawler')

const ENUM_LENGTH = 1
const SIZE_LENGTH = 2
const ADDRESS_SIZE_LENGTH = 1

class CrawlResponse extends Uint8Array {
  constructor(
    arr:
      | {
          bytes: ArrayBuffer
          offset: number
        }
      | undefined
      | null,
    struct?: {
      status: CrawlStatus
      addresses?: Multiaddr[]
    }
  ) {
    if (arr == null && struct == null) {
      throw Error(`Could not create CrawlResponse. Either provide an array or a data object.`)
    }

    if (arr != null) {
      const length = u8aToNumber(new Uint8Array(arr.bytes, arr.offset, SIZE_LENGTH))

      if (length < CrawlResponse.SIZE) {
        throw Error(`Invalid length-prefix. Got ${u8aToHex(new Uint8Array(arr.bytes, arr.offset, SIZE_LENGTH))}`)
      }

      if (arr.bytes.byteLength < length + arr.offset) {
        throw Error(
          `Invalid Uint8Array. Expected ${length + arr.offset} elements but array has only ${arr.bytes.byteLength}`
        )
      }

      if (struct != null) {
        let addrsLength =
          struct.addresses.reduce((acc, addr) => acc + addr.bytes.length, 0) +
          struct.addresses.length * ADDRESS_SIZE_LENGTH

        if (addrsLength + CrawlResponse.SIZE > arr.bytes.byteLength + arr.offset) {
          throw Error(
            `Given multiaddrs do not fit into given array. Please provide an array that has ${
              addrsLength + CrawlResponse.SIZE - (arr.bytes.byteLength + arr.offset)
            } more elements`
          )
        }
      }

      super(arr.bytes, arr.offset, length)
    } else {
      let addrsLength =
        struct.addresses.reduce((acc, addr) => acc + addr.bytes.length, 0) +
        struct.addresses.length * ADDRESS_SIZE_LENGTH

      super(CrawlResponse.SIZE + addrsLength)

      this.set(toU8a(CrawlResponse.SIZE + addrsLength, SIZE_LENGTH), 0)
    }

    if (struct != null) {
      this.set(new Uint8Array([struct.status]), SIZE_LENGTH)

      let offset = CrawlResponse.SIZE
      for (let i = 0; i < struct.addresses.length; i++) {
        if (struct.addresses[i].bytes.length > (1 << (ADDRESS_SIZE_LENGTH * 8)) - 1) {
          throw Error(
            `Invalid Multiaddr. Multiaddr has length ${
              struct.addresses[i].bytes.length
            } but we accept only multiaddrs of length ${(1 << (ADDRESS_SIZE_LENGTH * 8)) - 1}`
          )
        }
        this.set(toU8a(struct.addresses[i].bytes.length, ADDRESS_SIZE_LENGTH), offset)
        this.set(struct.addresses[i].bytes, offset + ADDRESS_SIZE_LENGTH)

        offset += ADDRESS_SIZE_LENGTH + struct.addresses[i].bytes.length
      }
    }
  }

  slice(begin: number = 0, end?: number) {
    return this.subarray(begin, end)
  }

  subarray(begin: number = 0, end?: number): Uint8Array {
    return new Uint8Array(this.buffer, this.byteOffset + begin, end != null ? end - begin : undefined)
  }

  get sizeRaw(): Uint8Array {
    return this.subarray(0, SIZE_LENGTH)
  }

  get size(): number {
    return u8aToNumber(this.sizeRaw)
  }

  get statusRaw(): Uint8Array {
    return this.subarray(SIZE_LENGTH, SIZE_LENGTH + ENUM_LENGTH)
  }

  get status(): CrawlStatus {
    return u8aToNumber(this.statusRaw)
  }

  get addressesRaw(): Uint8Array {
    return this.subarray(ENUM_LENGTH)
  }

  get addresses(): Multiaddr[] {
    let result: Multiaddr[] = []

    let offset = CrawlResponse.SIZE
    let size = this.size

    while (offset < size) {
      const multiAddrSize = u8aToNumber(this.subarray(offset, offset + ADDRESS_SIZE_LENGTH))
      try {
        result.push(
          new Multiaddr(this.subarray(offset + ADDRESS_SIZE_LENGTH, offset + ADDRESS_SIZE_LENGTH + multiAddrSize))
        )
      } catch (err) {
        log(
          `Could not decode multiaddr from array ${u8aToHex(
            this.subarray(offset + ADDRESS_SIZE_LENGTH, offset + ADDRESS_SIZE_LENGTH + multiAddrSize)
          )}`
        )
      }
      offset += ADDRESS_SIZE_LENGTH + multiAddrSize
    }
    return result
  }

  static get SIZE() {
    return SIZE_LENGTH + ENUM_LENGTH
  }
}

export { CrawlResponse }
