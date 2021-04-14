import { u8aSplit, serializeToU8a, toU8a } from '@hoprnet/hopr-utils'
import { Address, Balance, Hash } from './primitives'
import { UINT256 } from '../types/solidity'
import BN from 'bn.js'

const sizes = [
  Address.SIZE,
  Address.SIZE,
  UINT256.SIZE,
  UINT256.SIZE,
  Balance.SIZE,
  Balance.SIZE,
  UINT256.SIZE,
  UINT256.SIZE,
  1,
  UINT256.SIZE,
  UINT256.SIZE,
  Hash.SIZE
]

export class Channel {
  constructor(
    public readonly self: Address,
    public readonly counterparty: Address,
    // Ticket epoch
    public readonly selfEpoch: UINT256,
    public readonly counterpartyEpoch: UINT256,
    public readonly selfBalance: Balance,
    public readonly counterpartyBalance: Balance,
    public readonly closureTime: BN,
    public readonly channelEpoch: UINT256,
    public readonly closureByPartyA: boolean,
    public readonly openedAt: BN,
    public readonly closedAt: BN,
    public readonly commitment: Hash
  ) {}

  static get SIZE(): number {
    return sizes.reduce((x, y) => x + y, 0)
  }

  static deserialize(arr: Uint8Array) {
    const items = u8aSplit(arr, sizes)
    const self = new Address(items[0])
    const counterparty = new Address(items[1])
    const selfEpoch = UINT256.deserialize(items[2])
    const counterpartyEpoch = UINT256.deserialize(items[3])
    const selfBalance = Balance.deserialize(items[4])
    const counterpartyBalance = Balance.deserialize(items[5])
    const closureTime = new BN(items[4])
    const channelEpoch = UINT256.deserialize(items[5])
    const closureByPartyA = Boolean(items[6][0])
    const openedAt = new BN(items[7])
    const closedAt = new BN(items[8])
    const commitment = new Hash(items[9])

    return new Channel(
      self,
      counterparty,
      selfEpoch,
      counterpartyEpoch,
      selfBalance,
      counterpartyBalance,
      closureTime,
      channelEpoch,
      closureByPartyA,
      openedAt,
      closedAt,
      commitment
    )
  }

  public serialize(): Uint8Array {
    return serializeToU8a([
      [this.self.serialize(), Address.SIZE],
      [this.counterparty.serialize(), Address.SIZE],
      [this.selfBalance.toUINT256().serialize(), UINT256.SIZE],
      [this.counterpartyBalance.toUINT256().serialize(), UINT256.SIZE],
      [new UINT256(this.closureTime).serialize(), UINT256.SIZE],
      [this.channelEpoch.serialize(), UINT256.SIZE],
      [toU8a(Number(this.closureByPartyA)), 1],
      [new UINT256(this.openedAt).serialize(), UINT256.SIZE],
      [new UINT256(this.closedAt).serialize(), UINT256.SIZE],
      [this.commitment.serialize(), Hash.SIZE]
    ])
  }

  public getStatus() {
    const status = this.channelEpoch.toBN().modn(10)
    if (status === 0) return 'CLOSED'
    else if (status === 1) return 'OPEN'
    else if (status === 2) return 'PENDING_TO_CLOSE'
    throw Error(`Status at ${status} does not exist`)
  }

  public getIteration() {
    return this.channelEpoch.toBN().addn(1).divn(10)
  }

  static generateId(self: Address, counterparty: Address) {
    let parties = self.sortPair(counterparty)
    return Hash.create(Buffer.concat(parties.map((x) => x.serialize())))
  }

  public getId() {
    const parties = this.self.sortPair(this.counterparty)
    return Channel.generateId(...parties)
  }
}
