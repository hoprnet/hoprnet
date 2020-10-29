import BN from 'bn.js'
import { UINT256 } from '../types/solidity'
import { BNE, Uint8ArrayE } from '../types/extended'

// @TODO: we should optimize this since it will use more storage than needed
class ChannelEntry extends Uint8ArrayE {
  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      blockNumber: BN
      transactionIndex: BN
      logIndex: BN
    }
  ) {
    if (arr == null) {
      super(ChannelEntry.SIZE)
    } else {
      super(arr.bytes, arr.offset, ChannelEntry.SIZE)
    }

    if (struct != null) {
      // we convert values to string because of this issue
      // https://github.com/indutny/bn.js/issues/206
      const blockNumber = new BNE(struct.blockNumber.toString())
      const transactionIndex = new BNE(struct.transactionIndex.toString())
      const logIndex = new BNE(struct.logIndex.toString())

      this.set(blockNumber.toU8a(UINT256.SIZE), this.blockNumberOffset - this.byteOffset)
      this.set(transactionIndex.toU8a(UINT256.SIZE), this.transactionIndexOffset - this.byteOffset)
      this.set(logIndex.toU8a(UINT256.SIZE), this.logIndexOffset - this.byteOffset)
    }
  }

  slice(begin = 0, end = ChannelEntry.SIZE): Uint8Array {
    return this.subarray(begin, end)
  }

  subarray(begin = 0, end = ChannelEntry.SIZE): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  get blockNumberOffset() {
    return this.byteOffset
  }

  get blockNumber() {
    return new BNE(new Uint8Array(this.buffer, this.blockNumberOffset, UINT256.SIZE))
  }

  get transactionIndexOffset() {
    return this.byteOffset + UINT256.SIZE
  }

  get transactionIndex() {
    return new BNE(new Uint8Array(this.buffer, this.transactionIndexOffset, UINT256.SIZE))
  }

  get logIndexOffset() {
    return this.byteOffset + UINT256.SIZE + UINT256.SIZE
  }

  get logIndex() {
    return new BNE(new Uint8Array(this.buffer, this.logIndexOffset, UINT256.SIZE))
  }

  static get SIZE(): number {
    return UINT256.SIZE * 3
  }
}

export default ChannelEntry
