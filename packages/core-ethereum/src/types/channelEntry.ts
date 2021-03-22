import { Types, ChannelStatus } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { UINT256 } from '../types/solidity'
import { Uint8ArrayE } from '../types/extended'

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
      this.set(struct.blockNumber.toBuffer('be', UINT256.SIZE), this.blockNumberOffset - this.byteOffset)
      this.set(struct.transactionIndex.toBuffer('be', UINT256.SIZE), this.transactionIndexOffset - this.byteOffset)
      this.set(struct.logIndex.toBuffer('be', UINT256.SIZE), this.logIndexOffset - this.byteOffset)
      this.set(struct.deposit.toBuffer('be', UINT256.SIZE), this.depositOffset - this.byteOffset)
      this.set(struct.partyABalance.toBuffer('be', UINT256.SIZE), this.partyABalanceOffset - this.byteOffset)
      this.set(struct.closureTime.toBuffer('be', UINT256.SIZE), this.closureTimeOffset - this.byteOffset)
      this.set(struct.stateCounter.toBuffer('be', UINT256.SIZE), this.stateCounterOffset - this.byteOffset)
      this.set(new Uint8Array([Number(struct.closureByPartyA)]), this.closureByPartyAOffset - this.byteOffset)
    }
  }

  slice(begin = 0, end = ChannelEntry.SIZE): Uint8Array {
    return this.subarray(begin, end)
  }

  subarray(begin = 0, end = ChannelEntry.SIZE): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
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

  get depositOffset(): number {
    return this.logIndexOffset + UINT256.SIZE
  }

  get deposit() {
    return new BN(new Uint8Array(this.buffer, this.depositOffset, UINT256.SIZE))
  }

  get partyABalanceOffset(): number {
    return this.depositOffset + UINT256.SIZE
  }

  get partyABalance() {
    return new BN(new Uint8Array(this.buffer, this.partyABalanceOffset, UINT256.SIZE))
  }

  get closureTimeOffset(): number {
    return this.partyABalanceOffset + UINT256.SIZE
  }

  get closureTime() {
    return new BN(new Uint8Array(this.buffer, this.closureTimeOffset, UINT256.SIZE))
  }

  get stateCounterOffset(): number {
    return this.closureTimeOffset + UINT256.SIZE
  }

  get stateCounter() {
    return new BN(new Uint8Array(this.buffer, this.stateCounterOffset, UINT256.SIZE))
  }

  get closureByPartyAOffset(): number {
    return this.stateCounterOffset + UINT256.SIZE
  }

  get closureByPartyA() {
    return Boolean(new Uint8Array(this.buffer, this.closureByPartyAOffset, 1)[0])
  }

  get status(): ChannelStatus {
    const status = Object.keys(ChannelStatus).find(k => ChannelStatus[k] == this.stateCounter.toNumber())
    if (!status) {
      throw Error("status like this doesn't exist")
    }
    return status
  }

  static get SIZE(): number {
    return UINT256.SIZE * 7 + 1
  }
}

export default ChannelEntry
