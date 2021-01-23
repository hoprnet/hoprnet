import type { Types } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { UINT256 } from '../types/solidity'
import { BNE, Uint8ArrayE } from '../types/extended'
import { ChannelStatus } from '../types/channel'

// @TODO: we should optimize this since it will use more storage than needed
// @TODO: redesign how we build classes like this
class ChannelEntry extends Uint8ArrayE implements Types.ChannelEntry {
  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      blockNumber: BN
      transactionIndex: BN
      logIndex: BN
      deposit: BN
      partyABalance: BN
      closureTime: BN
      stateCounter: BN
      closureByPartyA: boolean
    }
  ) {
    if (!arr) {
      super(ChannelEntry.SIZE)
    } else {
      super(arr.bytes, arr.offset, ChannelEntry.SIZE)
    }

    if (struct) {
      this.set(new BNE(struct.blockNumber).toU8a(UINT256.SIZE), this.byteOffset * 1 - this.byteOffset)
      this.set(new BNE(struct.transactionIndex).toU8a(UINT256.SIZE), this.byteOffset * 2 - this.byteOffset)
      this.set(new BNE(struct.logIndex).toU8a(UINT256.SIZE), this.byteOffset * 3 - this.byteOffset)
      this.set(new BNE(struct.deposit).toU8a(UINT256.SIZE), this.byteOffset * 4 - this.byteOffset)
      this.set(new BNE(struct.partyABalance).toU8a(UINT256.SIZE), this.byteOffset * 5 - this.byteOffset)
      this.set(new BNE(struct.closureTime).toU8a(UINT256.SIZE), this.byteOffset * 6 - this.byteOffset)
      this.set(new BNE(struct.stateCounter).toU8a(UINT256.SIZE), this.byteOffset * 7 - this.byteOffset)
      this.set([Number(struct.closureByPartyA)], this.byteOffset * 8 - this.byteOffset)
    }
  }

  slice(begin = 0, end = ChannelEntry.SIZE): Uint8Array {
    return this.subarray(begin, end)
  }

  subarray(begin = 0, end = ChannelEntry.SIZE): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  get blockNumber() {
    return new BNE(new Uint8Array(this.buffer, this.byteOffset * 1, UINT256.SIZE))
  }

  get transactionIndex() {
    return new BNE(new Uint8Array(this.buffer, this.byteOffset * 2, UINT256.SIZE))
  }

  get logIndex() {
    return new BNE(new Uint8Array(this.buffer, this.byteOffset * 3, UINT256.SIZE))
  }

  get deposit() {
    return new BNE(new Uint8Array(this.buffer, this.byteOffset * 4, UINT256.SIZE))
  }

  get partyABalance() {
    return new BNE(new Uint8Array(this.buffer, this.byteOffset * 5, UINT256.SIZE))
  }

  get closureTime() {
    return new BNE(new Uint8Array(this.buffer, this.byteOffset * 6, UINT256.SIZE))
  }

  get stateCounter() {
    return new BNE(new Uint8Array(this.buffer, this.byteOffset * 7, UINT256.SIZE))
  }

  get closureByPartyA() {
    return Boolean(new Uint8Array(this.buffer, this.byteOffset * 7 + 1, 1)[0])
  }

  get status() {
    const stateCounter = this.stateCounter
    const status = stateCounter.modn(10)

    if (status >= Object.keys(ChannelStatus).length) {
      throw Error("status like this doesn't exist")
    }

    if (status === ChannelStatus.UNINITIALISED) return 'UNINITIALISED'
    else if (status === ChannelStatus.FUNDING) return 'FUNDING'
    else if (status === ChannelStatus.OPEN) return 'OPEN'
    return 'PENDING'
  }

  static get SIZE(): number {
    return UINT256.SIZE * 7 + 1
  }
}

export default ChannelEntry
