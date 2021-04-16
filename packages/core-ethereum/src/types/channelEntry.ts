import { u8aSplit, serializeToU8a, u8aToNumber, stringToU8a } from '@hoprnet/hopr-utils'
import { Address, Balance, Hash } from './primitives'
import { UINT256 } from '../types/solidity'
import { Channel } from '..'
import type { Event } from '../indexer/types'
import BN from 'bn.js'

export type ChannelStatus = 'CLOSED' | 'OPEN' | 'PENDING_TO_CLOSE'

function numberToChannelStatus(i: number): ChannelStatus {
  if (i === 0) return 'CLOSED'
  else if (i === 1) return 'OPEN'
  else if (i === 2) return 'PENDING_TO_CLOSE'
  throw Error(`Status at ${status} does not exist`)
}

function u8aToChannelStatus(arr: Uint8Array): ChannelStatus {
  return numberToChannelStatus(u8aToNumber(arr) as number)
}

function channelStatusToU8a(c: ChannelStatus): Uint8Array {
  if (c == 'CLOSED') return Uint8Array.of(0)
  if (c == 'OPEN') return Uint8Array.of(1)
  return Uint8Array.of(2)
}

// TODO, find a better way to do this.
const components = [
  Address,
  Address,
  Balance,
  Balance,
  Hash,
  Hash,
  UINT256,
  UINT256,
  UINT256,
  UINT256,
  { name: 'channelStatus', SIZE: 32, deserialize: u8aToChannelStatus },
  UINT256,
  UINT256,
  { name: 'closureByPartyA', SIZE: 1, deserialize: () => {} }
]

class ChannelEntry {
  constructor(
    public readonly partyA: Address,
    public readonly partyB: Address,
    public readonly partyABalance: Balance,
    public readonly partyBBalance: Balance,
    public readonly commitmentPartyA: Hash,
    public readonly commitmentPartyB: Hash,
    public readonly partyATicketEpoch: UINT256,
    public readonly partyBTicketEpoch: UINT256,
    public readonly partyATicketIndex: UINT256,
    public readonly partyBTicketIndex: UINT256,
    public readonly status: ChannelStatus,
    public readonly channelEpoch: UINT256,
    public readonly closureTime: UINT256,
    public readonly closureByPartyA: boolean
  ) {}

  static get SIZE(): number {
    return components.map((x) => x.SIZE).reduce((x, y) => x + y, 0)
  }

  static deserialize(arr: Uint8Array) {
    const items = u8aSplit(
      arr,
      components.map((x) => x.SIZE)
    )
    const params = items.map((x, i) => components[i].deserialize(x))
    // @ts-ignore //TODO
    return new ChannelEntry(...params)
  }

  static fromSCEvent(event: Event<'ChannelUpdate'>): ChannelEntry {
    const data = event.args
    const rawChannel = data[2]
    return new ChannelEntry(
      Address.fromString(data.partyA),
      Address.fromString(data.partyB),
      new Balance(new BN(rawChannel[0].toString())),
      new Balance(new BN(rawChannel[1].toString())),
      new Hash(stringToU8a(rawChannel[2])),
      new Hash(stringToU8a(rawChannel[3])),
      new UINT256(new BN(rawChannel[4].toString())),
      new UINT256(new BN(rawChannel[5].toString())),
      new UINT256(new BN(rawChannel[6].toString())),
      new UINT256(new BN(rawChannel[7].toString())),
      numberToChannelStatus(rawChannel[8]),
      new UINT256(new BN(rawChannel[9].toString())),
      new UINT256(new BN(rawChannel[10].toString())),
      Boolean(rawChannel[11])
    )
  }

  public serialize(): Uint8Array {
    return serializeToU8a([
      [this.partyA.serialize(), Address.SIZE],
      [this.partyB.serialize(), Address.SIZE],
      [this.partyABalance.serialize(), Balance.SIZE],
      [this.partyBBalance.serialize(), Balance.SIZE],
      [this.commitmentPartyA.serialize(), Hash.SIZE],
      [this.commitmentPartyB.serialize(), Hash.SIZE],
      [this.partyATicketEpoch.serialize(), UINT256.SIZE],
      [this.partyBTicketEpoch.serialize(), UINT256.SIZE],
      [this.partyATicketIndex.serialize(), UINT256.SIZE],
      [this.partyBTicketIndex.serialize(), UINT256.SIZE],
      [channelStatusToU8a(this.status), 32],
      [this.channelEpoch.serialize(), UINT256.SIZE],
      [this.closureTime.serialize(), UINT256.SIZE],
      [Uint8Array.of(Number(this.closureByPartyA)), 1]
    ])
  }

  public getId() {
    return Channel.generateId(this.partyA, this.partyB)
  }
}

export default ChannelEntry
