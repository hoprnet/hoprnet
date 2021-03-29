import type { Types } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { u8aSplit, serializeToU8a, toU8a } from '@hoprnet/hopr-utils'
import { Address } from '.' // TODO: cyclic
import { UINT256 } from '../types/solidity'
import { ChannelStatus } from '../types/channel'
import { stateCounterToStatus, stateCounterToIteration, getId } from '../utils'

// TODO: optimize storage
class ChannelEntry implements Types.ChannelEntry {
  constructor(
    public readonly partyA: Address,
    public readonly partyB: Address,
    public readonly deposit: BN,
    public readonly partyABalance: BN,
    public readonly closureTime: BN,
    public readonly stateCounter: BN,
    public readonly closureByPartyA: boolean,
    public readonly openedAt: BN,
    public readonly closedAt: BN
  ) {}

  // TODO: implement .fromObject function

  static get SIZE(): number {
    return (
      Address.SIZE +
      Address.SIZE +
      UINT256.SIZE +
      UINT256.SIZE +
      UINT256.SIZE +
      UINT256.SIZE +
      1 +
      UINT256.SIZE +
      UINT256.SIZE
    )
  }

  static deserialize(arr: Uint8Array) {
    const items = u8aSplit(arr, [
      Address.SIZE,
      Address.SIZE,
      UINT256.SIZE,
      UINT256.SIZE,
      UINT256.SIZE,
      UINT256.SIZE,
      1,
      UINT256.SIZE,
      UINT256.SIZE
    ])
    const partyA = new Address(items[0])
    const partyB = new Address(items[1])
    const deposit = new BN(items[2])
    const partyABalance = new BN(items[3])
    const closureTime = new BN(items[4])
    const stateCounter = new BN(items[5])
    const closureByPartyA = Boolean(items[6][0])
    const openedAt = new BN(items[7])
    const closedAt = new BN(items[8])

    return new ChannelEntry(
      partyA,
      partyB,
      deposit,
      partyABalance,
      closureTime,
      stateCounter,
      closureByPartyA,
      openedAt,
      closedAt
    )
  }

  public serialize(): Uint8Array {
    return serializeToU8a([
      [this.partyA.serialize(), Address.SIZE],
      [this.partyB.serialize(), Address.SIZE],
      [new UINT256(this.deposit).serialize(), UINT256.SIZE],
      [new UINT256(this.partyABalance).serialize(), UINT256.SIZE],
      [new UINT256(this.closureTime).serialize(), UINT256.SIZE],
      [new UINT256(this.stateCounter).serialize(), UINT256.SIZE],
      [toU8a(Number(this.closureByPartyA)), 1],
      [new UINT256(this.openedAt).serialize(), UINT256.SIZE],
      [new UINT256(this.closedAt).serialize(), UINT256.SIZE]
    ])
  }

  public getStatus() {
    const status = stateCounterToStatus(this.stateCounter)

    if (status >= Object.keys(ChannelStatus).length) {
      throw Error("status like this doesn't exist")
    }

    if (status === ChannelStatus.CLOSED) return 'CLOSED'
    else if (status === ChannelStatus.PENDING_TO_CLOSE) return 'PENDING_TO_CLOSE'
    return 'OPEN'
  }

  public getIteration() {
    return stateCounterToIteration(this.stateCounter)
  }

  public getChannelId() {
    return getId(this.partyA, this.partyB)
  }
}

export default ChannelEntry
