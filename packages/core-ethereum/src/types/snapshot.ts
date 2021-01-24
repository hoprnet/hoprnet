import BN from 'bn.js'
import { UINT256 } from '../types/solidity'
import { Uint8ArrayE } from '../types/extended'

// @TODO: redesign how we build classes like this
/**
 * Represents a snapshot in the blockchain.
 */
class Snapshot extends Uint8ArrayE {
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
    if (!arr) {
      super(Snapshot.SIZE)
    } else {
      super(arr.bytes, arr.offset, Snapshot.SIZE)
    }

    if (struct) {
      this.set(struct.blockNumber.toBuffer('be', UINT256.SIZE), this.blockNumberOffset - this.byteOffset)
      this.set(struct.transactionIndex.toBuffer('be', UINT256.SIZE), this.transactionIndexOffset - this.byteOffset)
      this.set(struct.logIndex.toBuffer('be', UINT256.SIZE), this.logIndexOffset - this.byteOffset)
    }
  }

  get blockNumberOffset(): number {
    return this.byteOffset
  }

  get blockNumber() {
    return new BN(new Uint8Array(this.buffer, this.blockNumberOffset, UINT256.SIZE))
  }

  get transactionIndexOffset(): number {
    return this.blockNumberOffset + UINT256.SIZE
  }

  get transactionIndex() {
    return new BN(new Uint8Array(this.buffer, this.transactionIndexOffset, UINT256.SIZE))
  }

  get logIndexOffset(): number {
    return this.transactionIndexOffset + UINT256.SIZE
  }

  get logIndex() {
    return new BN(new Uint8Array(this.buffer, this.logIndexOffset, UINT256.SIZE))
  }

  static get SIZE(): number {
    return UINT256.SIZE * 3
  }
}

export default Snapshot
