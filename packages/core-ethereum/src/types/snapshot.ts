import BN from 'bn.js'
import { u8aSplit, serializeToU8a } from '@hoprnet/hopr-utils'
import { UINT256 } from '../types/solidity'

/**
 * Represents a snapshot in the blockchain.
 */
class Snapshot {
  constructor(public readonly blockNumber: BN, public readonly transactionIndex: BN, public readonly logIndex: BN) {}

  static deserialize(arr: Uint8Array) {
    const items = u8aSplit(arr, [UINT256.SIZE, UINT256.SIZE, UINT256.SIZE])
    const blockNumber = new BN(items[0])
    const transactionIndex = new BN(items[1])
    const logIndex = new BN(items[2])

    return new Snapshot(blockNumber, transactionIndex, logIndex)
  }

  public serialize(): Uint8Array {
    return serializeToU8a([
      [new UINT256(this.blockNumber.toString()).toU8a(), UINT256.SIZE],
      [new UINT256(this.transactionIndex.toString()).toU8a(), UINT256.SIZE],
      [new UINT256(this.logIndex.toString()).toU8a(), UINT256.SIZE]
    ])
  }

  static get SIZE(): number {
    return UINT256.SIZE * 3
  }
}

export default Snapshot
