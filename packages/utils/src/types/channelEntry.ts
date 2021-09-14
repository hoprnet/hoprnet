import { u8aConcat, u8aSplit, serializeToU8a, u8aToNumber, stringToU8a } from '..'
import { Address, Balance, Hash, PublicKey } from './primitives'
import { UINT256 } from './solidity'
import BN from 'bn.js'
import chalk from 'chalk'

export enum ChannelStatus {
  Closed = 0,
  WaitingForCommitment = 1,
  Open = 2,
  PendingToClose = 3
}

export function generateChannelId(source: Address, destination: Address) {
  return Hash.create(u8aConcat(source.serialize(), destination.serialize()))
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
  return Uint8Array.of(c)
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
  public sourcePubKey?: PublicKey
  public destinationPubKey?: PublicKey

  constructor(
    public readonly source: Address,
    public readonly destination: Address,
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

  static async fromSCEvent(event: any): Promise<ChannelEntry> {
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
      `ChannelEntry   (${chalk.yellow(this.getId().toHex())}):\n` +
      `  source:       ${chalk.yellow(this.source.toHex())}\n` +
      `  destination:  ${chalk.yellow(this.destination.toHex())}\n` +
      `  balance:      ${this.balance.toFormattedString()}\n` +
      `  commitment:   ${this.commitment.toHex()}\n` +
      `  ticketEpoch:  ${this.ticketEpoch.toBN().toString(10)}\n` +
      `  ticketIndex:  ${this.ticketIndex.toBN().toString(10)}\n` +
      `  status:       ${chalk.green(this.status)}\n` +
      `  channelEpoch: ${this.channelEpoch.toBN().toString(10)}\n` +
      `  closureTime:  ${this.closureTime.toBN().toString(10)}\n`
    )
  }

  public getId() {
    return generateChannelId(this.source, this.destination)
  }

  public async findPublicKeys(getPublicKeyOf: (addr: Address) => Promise<PublicKey>): Promise<{
    sourcePubKey?: PublicKey
    destinationPubKey?: PublicKey
  }> {
    if (!this.sourcePubKey) {
      try {
        this.sourcePubKey = await getPublicKeyOf(this.source)
      } catch {}
    }
    if (!this.destinationPubKey) {
      try {
        this.destinationPubKey = await getPublicKeyOf(this.destination)
      } catch {}
    }

    return {
      sourcePubKey: this.sourcePubKey,
      destinationPubKey: this.destinationPubKey
    }
  }

  public async hasPublicKeys(getPublicKeyOf: (addr: Address) => Promise<PublicKey>): Promise<boolean> {
    const { sourcePubKey, destinationPubKey } = await this.findPublicKeys(getPublicKeyOf)
    return !!sourcePubKey && !!destinationPubKey
  }

  public static createMock(): ChannelEntry {
    const addr = PublicKey.createMock().toAddress()
    return new ChannelEntry(
      addr,
      addr,
      new Balance(new BN(1)),
      Hash.create(),
      new UINT256(new BN(1)),
      new UINT256(new BN(1)),
      ChannelStatus.Closed,
      new UINT256(new BN(1)),
      new UINT256(new BN(1))
    )
  }
}
