import type { Types } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { UINT256 } from '../types/solidity'
import { Uint8ArrayE } from '../types/extended'
import { BYTES27_LENGTH } from '../constants'

// @TODO: redesign how we build classes like this
class AccountEntry extends Uint8ArrayE implements Types.AccountEntry {
  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      blockNumber: BN
      transactionIndex: BN
      logIndex: BN
      hashedSecret: Uint8Array
      counter: BN
    }
  ) {
    if (!arr) {
      super(AccountEntry.SIZE)
    } else {
      super(arr.bytes, arr.offset, AccountEntry.SIZE)
    }

    if (struct) {
      this.set(struct.blockNumber.toBuffer('be', UINT256.SIZE), this.blockNumberOffset - this.byteOffset)
      this.set(struct.transactionIndex.toBuffer('be', UINT256.SIZE), this.transactionIndexOffset - this.byteOffset)
      this.set(struct.logIndex.toBuffer('be', UINT256.SIZE), this.logIndexOffset - this.byteOffset)
      this.set(struct.hashedSecret, this.hashedSecretOffset - this.byteOffset)
      this.set(struct.counter.toBuffer('be', UINT256.SIZE), this.counterOffset - this.byteOffset)
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

  get hashedSecretOffset(): number {
    return this.logIndexOffset + UINT256.SIZE
  }

  get hashedSecret() {
    return new Uint8Array(this.buffer, this.hashedSecretOffset, BYTES27_LENGTH)
  }

  get counterOffset(): number {
    return this.hashedSecretOffset + BYTES27_LENGTH
  }

  get counter() {
    return new BN(new Uint8Array(this.buffer, this.counterOffset, UINT256.SIZE))
  }

  static get SIZE(): number {
    return UINT256.SIZE * 4 + BYTES27_LENGTH
  }
}

export default AccountEntry
