import { u8aSplit, serializeToU8a, u8aToNumber } from '@hoprnet/hopr-utils'
import { Address, Balance, Hash } from './primitives'
import { UINT256 } from '../types/solidity'
import { Channel } from '..'

export type ChannelStatus = 'CLOSED' | 'OPEN' | 'PENDING_TO_CLOSE'

function u8aToChannelStatus(arr: Uint8Array): ChannelStatus {
  const i = u8aToNumber(arr)
  if (i === 0) return 'CLOSED'
  else if (i === 1) return 'OPEN'
  else if (i === 2) return 'PENDING_TO_CLOSE'
  throw Error(`Status at ${status} does not exist`)
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
  { name: 'channelStatus', SIZE: 32, deserialize: u8aToChannelStatus},
  UINT256,
  UINT256,
  { name: 'closureByPartyA', SIZE: 1, deserialize: () => {}}
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
    return components.map(x => x.SIZE).reduce((x, y) => x + y, 0)
  }

  static deserialize(arr: Uint8Array) {
    const items = u8aSplit(arr, components.map(x => x.SIZE))
    const params = items.map((x, i) => components[i].deserialize(x))
    // @ts-ignore //TODO
    return new ChannelEntry(...params)
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
      [this.closureTime: UINT256.SIZE],
      [this.closureByPartyA, 1]
    ])
  }

  public getId() {
    return Channel.generateId(this.partyA, this.partyB)
  }
}

export default ChannelEntry
