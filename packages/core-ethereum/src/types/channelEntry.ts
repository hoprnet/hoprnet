import BN from 'bn.js'
import { u8aConcat } from '@hoprnet/hopr-utils'
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
    if (arr != null && struct == null) {
      super(arr.bytes, arr.offset, ChannelEntry.SIZE)
    } else if (arr == null && struct != null) {
      // we convert values to string because of this issue
      // https://github.com/indutny/bn.js/issues/206
      const blockNumber = new BNE(struct.blockNumber.toString())
      const transactionIndex = new BNE(struct.transactionIndex.toString())
      const logIndex = new BNE(struct.logIndex.toString())

      super(
        u8aConcat(blockNumber.toU8a(UINT256.SIZE), transactionIndex.toU8a(UINT256.SIZE), logIndex.toU8a(UINT256.SIZE))
      )
    } else {
      throw Error(`Invalid constructor arguments.`)
    }
  }

  get blockNumber() {
    return new BNE(this.subarray(0, UINT256.SIZE))
  }

  get transactionIndex() {
    return new BNE(this.subarray(UINT256.SIZE, UINT256.SIZE * 2))
  }

  get logIndex() {
    return new BNE(this.subarray(UINT256.SIZE * 2, UINT256.SIZE * 3))
  }

  static get SIZE(): number {
    return UINT256.SIZE * 3
  }
}

export default ChannelEntry
