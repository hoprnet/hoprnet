import { Types } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import AccountId from './accountId'
import { UINT256 } from '../types/solidity'
import { ChannelStatus } from '../types/channel'
import { stateCounterToStatus, stateCounterToIteration } from '../utils'

// TODO: optimize storage
class ChannelEntry implements Types.ChannelEntry {
  constructor(
    public readonly parties: [AccountId, AccountId],
    public readonly deposit: BN,
    public readonly partyABalance: BN,
    public readonly closureTime: BN,
    public readonly stateCounter: BN,
    public readonly closureByPartyA: boolean
  ) {}

  static deserialize(arr: Uint8Array) {
    const items = u8aSplit(arr, [
      AccountId.SIZE,
      AccountId.SIZE,
      UINT256.SIZE,
      UINT256.SIZE,
      UINT256.SIZE,
      UINT256.SIZE,
      1
    ])
    const parties: [AccountId, AccountId] = [new AccountId(items[0]), new AccountId(items[1])]
    const deposit = new BN(items[2])
    const partyABalance = new BN(items[3])
    const closureTime = new BN(items[4])
    const stateCounter = new BN(items[5])
    const closureByPartyA = Boolean(items[6][0])

    return new ChannelEntry(parties, deposit, partyABalance, closureTime, stateCounter, closureByPartyA)
  }

  public serialize(): Uint8Array {
    return serializeToU8a([
      [this.parties[0], AccountId.SIZE],
      [this.parties[1], AccountId.SIZE],
      [this.deposit.toBuffer(), UINT256.SIZE],
      [this.partyABalance.toBuffer(), UINT256.SIZE],
      [this.closureTime.toBuffer(), UINT256.SIZE],
      [this.stateCounter.toBuffer(), UINT256.SIZE],
      [[Number(this.closureByPartyA)], 1]
    ])
  }

  public getStatus() {
    const status = stateCounterToStatus(this.stateCounter.toNumber())

    if (status >= Object.keys(ChannelStatus).length) {
      throw Error("status like this doesn't exist")
    }

    if (status === ChannelStatus.UNINITIALISED) return 'UNINITIALISED'
    else if (status === ChannelStatus.FUNDED) return 'FUNDED'
    else if (status === ChannelStatus.OPEN) return 'OPEN'
    return 'PENDING'
  }

  public getIteration() {
    return stateCounterToIteration(this.stateCounter.toNumber())
  }
}

export default ChannelEntry
