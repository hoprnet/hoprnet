import { u8aSplit, serializeToU8a, u8aToNumber, stringToU8a } from '..'
import { Address, Balance, Hash } from './primitives'
import { UINT256 } from './solidity'
import BN from 'bn.js'
import chalk from 'chalk'

export enum ChannelStatus {
  Closed = 'CLOSED',
  WaitingForCommitment = 'WAITING_FOR_COMMITMENT',
  Open = 'OPEN',
  PendingToClose = 'PENDING_TO_CLOSE'
}

export function generateChannelId(self: Address, counterparty: Address) {
  let parties = self.sortPair(counterparty)
  return Hash.create(Buffer.concat(parties.map((x) => x.serialize())))
}

function numberToChannelStatus(i: number): ChannelStatus {
  switch (i) {
    case 0:
      return ChannelStatus.Closed
    case 1: 
      return ChannelStatus.WaitingForCommitment
    case 2:
      return ChannelStatus.Open
    case 3:
      return ChannelStatus.PendingToClose
    default:
      throw Error(`Status at ${status} does not exist`)
  }
}

function u8aToChannelStatus(arr: Uint8Array): ChannelStatus {
  return numberToChannelStatus(u8aToNumber(arr) as number)
}

function channelStatusToU8a(c: ChannelStatus): Uint8Array {
  switch (c) {
    case 'CLOSED':
      return Uint8Array.of(0)
    case 'WAITING_FOR_COMMITMENT':
      return Uint8Array.of(1)
    case 'OPEN':
      return Uint8Array.of(2)
    case 'PENDING_TO_CLOSE':
      return Uint8Array.of(3)
    default:
      throw Error(`Invalid status. Got ${c}`)
  }
}

// TODO, find a better way to do this.
const components = [
  Address,
  Address,
  Balance,
  Hash,
  UINT256,
  UINT256,
  { name: 'channelStatus', SIZE: 1, deserialize: u8aToChannelStatus },
  UINT256,
  UINT256,
]

export class ChannelEntry {
  constructor(
    public readonly source: Address,
    public readonly destination: Address,
    public readonly balance: Balance,
    public readonly commitment: Hash,
    public readonly ticketEpoch: UINT256,
    public readonly ticketIndex: UINT256,
    public readonly status: ChannelStatus,
    public readonly channelEpoch: UINT256,
    public readonly closureTime: UINT256,
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

  static fromSCEvent(event: any): ChannelEntry {
    // TODO type
    const { source, destination, newState } = event.args
    return new ChannelEntry(
      Address.fromString(source),
      Address.fromString(destination),
      new Balance(new BN(newState.balance.toString())),
      new Hash(stringToU8a(newState.commitment)),
      new UINT256(new BN(newState.ticketEpoch.toString())),
      new UINT256(new BN(newState.ticketIndex.toString())),
      numberToChannelStatus(newState.status),
      new UINT256(new BN(newState.channelEpoch.toString())),
      new UINT256(new BN(newState.closureTime.toString()))
    )
  }

  public serialize(): Uint8Array {
    return serializeToU8a([
      [this.source.serialize(), Address.SIZE],
      [this.destination.serialize(), Address.SIZE],
      [this.balance.serialize(), Balance.SIZE],
      [this.commitmentPartyA.serialize(), Hash.SIZE],
      [this.commitmentPartyB.serialize(), Hash.SIZE],
      [this.partyATicketEpoch.serialize(), UINT256.SIZE],
      [this.partyBTicketEpoch.serialize(), UINT256.SIZE],
      [this.partyATicketIndex.serialize(), UINT256.SIZE],
      [this.partyBTicketIndex.serialize(), UINT256.SIZE],
      [channelStatusToU8a(this.status), 1],
      [this.channelEpoch.serialize(), UINT256.SIZE],
      [this.closureTime.serialize(), UINT256.SIZE],
      [Uint8Array.of(Number(this.closureByPartyA)), 1]
    ])
  }

  toString() {
    return (
      // prettier-ignore
      `ChannelEntry (${chalk.yellow(this.getId().toHex())}):\n` +
      `  partyA:            ${chalk.yellow(this.partyA.toHex())}\n` +
      `  partyB:            ${chalk.yellow(this.partyB.toHex())}\n` +
      `  balanceA:          ${this.partyABalance.toFormattedString()}\n` +
      `  balanceB:          ${this.partyBBalance.toFormattedString()}\n` +
      `  commitmentA:       ${this.commitmentPartyA.toHex()}\n` +
      `  commitmentB:       ${this.commitmentPartyB.toHex()}\n` +
      `  partyATicketEpoch: ${this.partyATicketEpoch.toBN().toString(10)}\n` +
      `  partyBTicketEpoch: ${this.partyBTicketEpoch.toBN().toString(10)}\n` +
      `  partyBTicketIndex: ${this.partyATicketIndex.toBN().toString(10)}\n` +
      `  partyBTicketIndex: ${this.partyBTicketIndex.toBN().toString(10)}\n` +
      `  status:            ${chalk.green(this.status)}\n` +
      `  channelEpoch:      ${this.channelEpoch.toBN().toString(10)}\n` +
      `  closureTime:       ${this.closureTime.toBN().toString(10)}\n` +
      `  closedByA:         ${chalk.blue(this.closureByPartyA.toString())}\n`
    )
  }

  public getId() {
    return generateChannelId(this.partyA, this.partyB)
  }

  public totalBalance(): Balance {
    return new Balance(this.partyABalance.toBN().add(this.partyBBalance.toBN()))
  }

  public ticketEpochFor(addr: Address): UINT256 {
    if (addr.eq(this.partyA)) {
      return this.partyATicketEpoch
    }
    if (addr.eq(this.partyB)) {
      return this.partyBTicketEpoch
    }
    throw new Error('Wrong addr for this channel')
  }

  public ticketIndexFor(addr: Address): UINT256 {
    if (addr.eq(this.partyA)) {
      return this.partyATicketIndex
    }
    if (addr.eq(this.partyB)) {
      return this.partyBTicketIndex
    }
    throw new Error('Wrong addr for this channel')
  }

  public commitmentFor(addr: Address): Hash {
    if (addr.eq(this.partyA)) {
      return this.commitmentPartyA
    }
    if (addr.eq(this.partyB)) {
      return this.commitmentPartyB
    }
    throw new Error('Wrong addr for this channel')
  }
}
