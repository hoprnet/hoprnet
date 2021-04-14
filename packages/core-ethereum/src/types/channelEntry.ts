import { u8aSplit, serializeToU8a, toU8a } from '@hoprnet/hopr-utils'
import { Address, Balance, Hash } from './primitives'
import { UINT256 } from '../types/solidity'
import BN from 'bn.js'

export class Channel {
  constructor(
    public readonly self: Address,
    public readonly counterparty: Address,
    public readonly deposit: BN,
    public readonly partyABalance: BN,
    public readonly closureTime: BN,
    public readonly stateCounter: BN,
    public readonly closureByPartyA: boolean,
    public readonly openedAt: BN,
    public readonly closedAt: BN,
    public readonly commitment: Hash
  ) {}

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
      UINT256.SIZE +
      Hash.SIZE
    )
  }

  static fromObject(obj: {
    partyA: Address
    partyB: Address
    deposit: BN
    partyABalance: BN
    closureTime: BN
    stateCounter: BN
    closureByPartyA: boolean
    openedAt: BN
    closedAt: BN
    commitment: Hash
  }) {
    return new Channel(
      obj.partyA,
      obj.partyB,
      obj.deposit,
      obj.partyABalance,
      obj.closureTime,
      obj.stateCounter,
      obj.closureByPartyA,
      obj.openedAt,
      obj.closedAt,
      obj.commitment
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
      UINT256.SIZE,
      Hash.SIZE
    ])
    const self = new Address(items[0])
    const counterparty = new Address(items[1])
    const deposit = new BN(items[2])
    const partyABalance = new BN(items[3])
    const closureTime = new BN(items[4])
    const stateCounter = new BN(items[5])
    const closureByPartyA = Boolean(items[6][0])
    const openedAt = new BN(items[7])
    const closedAt = new BN(items[8])
    const commitment = new Hash(items[9])

    return new Channel(
      self,
      counterparty,
      deposit,
      partyABalance,
      closureTime,
      stateCounter,
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
      [new UINT256(this.deposit).serialize(), UINT256.SIZE],
      [new UINT256(this.partyABalance).serialize(), UINT256.SIZE],
      [new UINT256(this.closureTime).serialize(), UINT256.SIZE],
      [new UINT256(this.stateCounter).serialize(), UINT256.SIZE],
      [toU8a(Number(this.closureByPartyA)), 1],
      [new UINT256(this.openedAt).serialize(), UINT256.SIZE],
      [new UINT256(this.closedAt).serialize(), UINT256.SIZE],
      [this.commitment.serialize(), Hash.SIZE]
    ])
  }

  public getStatus() {
    const status = this.stateCounter.modn(10)

    if (status === 0) return 'CLOSED'
    else if (status === 1) return 'OPEN'
    else if (status === 2) return 'PENDING_TO_CLOSE'
    throw Error(`Status at ${status} does not exist`)
  }

  public getIteration() {
    return new BN(String(Math.ceil((this.stateCounter.toNumber() + 1) / 10)))
  }

  static generateId(self: Address, counterparty: Address) {
    let parties = self.sortPair(counterparty)
    return Hash.create(Buffer.concat(parties.map((x) => x.serialize())))
  }

  public getId() {
    const parties = this.self.sortPair(this.counterparty)
    return Channel.generateId(...parties)
  }

  public getBalances() {
    return {
      partyA: new Balance(this.partyABalance),
      partyB: new Balance(this.deposit.sub(this.partyABalance))
    }
  }
}
