import { u8aSplit, serializeToU8a, u8aToNumber, stringToU8a } from '..'
import { Address, Balance, Hash, PublicKey } from './primitives'
import { UINT256 } from './solidity'
import BN from 'bn.js'
import chalk from 'chalk'

export enum ChannelStatus {
  Closed = 'CLOSED',
  WaitingForCommitment = 'WAITING_FOR_COMMITMENT',
  Open = 'OPEN',
  PendingToClose = 'PENDING_TO_CLOSE'
}

export function generateChannelId(source: Address, destination: Address) {
  return Hash.create(Buffer.concat([source.serialize(), destination.serialize()]))
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
  UINT256
]

export class ChannelEntry {
  constructor(
    public readonly source: PublicKey,
    public readonly destination: PublicKey,
    public readonly balance: Balance,
    public readonly commitment: Hash,
    public readonly ticketEpoch: UINT256,
    public readonly ticketIndex: UINT256,
    public readonly status: ChannelStatus,
    public readonly channelEpoch: UINT256,
    public readonly closureTime: UINT256
  ) {}

  static get SIZE(): number {
    return components.map((x) => x.SIZE).reduce((x, y) => x + y, 0)
  }

  static deserialize(arr: Uint8Array) {
    const items = u8aSplit(
      arr,
      components.map((x) => x.SIZE)
    )

    const params = items.map((x, i) => components[i].deserialize(x)) as ConstructorParameters<typeof ChannelEntry>

    return new ChannelEntry(...params)
  }

  static async fromSCEvent(event: any, keyFor: (a: Address) => Promise<PublicKey>): Promise<ChannelEntry> {
    // TODO type
    const { source, destination, newState } = event.args
    return new ChannelEntry(
      await keyFor(Address.fromString(source)),
      await keyFor(Address.fromString(destination)),
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
      [this.commitment.serialize(), Hash.SIZE],
      [this.ticketEpoch.serialize(), UINT256.SIZE],
      [this.ticketIndex.serialize(), UINT256.SIZE],
      [channelStatusToU8a(this.status), 1],
      [this.channelEpoch.serialize(), UINT256.SIZE],
      [this.closureTime.serialize(), UINT256.SIZE]
    ])
  }

  toString() {
    return (
      // prettier-ignore
      `ChannelEntry (${chalk.yellow(this.getId().toHex())}):\n` +
      `  source:            ${chalk.yellow(this.source.toHex())}\n` +
      `  destination:       ${chalk.yellow(this.destination.toHex())}\n` +
      `  balance:           ${this.balance.toFormattedString()}\n` +
      `  commitment:        ${this.commitment.toHex()}\n` +
      `  ticketEpoch:       ${this.ticketEpoch.toBN().toString(10)}\n` +
      `  ticketIndex:       ${this.ticketIndex.toBN().toString(10)}\n` +
      `  status:            ${chalk.green(this.status)}\n` +
      `  channelEpoch:      ${this.channelEpoch.toBN().toString(10)}\n` +
      `  closureTime:       ${this.closureTime.toBN().toString(10)}\n`
    )
  }

  public getId() {
    return generateChannelId(this.source.toAddress(), this.destination.toAddress())
  }
}
