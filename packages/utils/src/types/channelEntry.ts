import { u8aSplit, serializeToU8a, u8aToNumber, stringToU8a } from '../index.js'
import { Address, Balance, Hash } from './primitives.js'
import { PublicKey } from './publicKey.js'
import { UINT256 } from './solidity.js'
import BN from 'bn.js'
import chalk from 'chalk'
import type { BigNumberish } from 'ethers'

import { ChannelStatus } from '../../../core/lib/core_types.js'
export { ChannelStatus } from '../../../core/lib/core_types.js'

export function generateChannelId(source: Address, destination: Address) {
  return Hash.create(Uint8Array.from([...source.serialize(), ...destination.serialize()]))
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
      throw Error(`Status at ${i} does not exist`)
  }
}

export function channelStatusToString(status: ChannelStatus): string {
  switch (status) {
    case ChannelStatus.Closed:
      return 'Closed'
    case ChannelStatus.WaitingForCommitment:
      return 'WaitingForCommitment'
    case ChannelStatus.Open:
      return 'Open'
    case ChannelStatus.PendingToClose:
      return 'PendingToClose'
    default:
      throw Error(`Status ${status} does not exist`)
  }
}

function u8aToChannelStatus(arr: Uint8Array): ChannelStatus {
  return numberToChannelStatus(u8aToNumber(arr) as number)
}

function channelStatusToU8a(c: ChannelStatus): Uint8Array {
  return Uint8Array.of(c)
}

type ChannelUpdateEvent = {
  args: {
    source: string
    destination: string
    newState: {
      balance: BigNumberish
      commitment: string
      ticketEpoch: BigNumberish
      ticketIndex: BigNumberish
      status: number
      channelEpoch: BigNumberish
      closureTime: BigNumberish
    }
  }
}

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
    return (
      PublicKey.SIZE_UNCOMPRESSED +
      PublicKey.SIZE_UNCOMPRESSED +
      Balance.SIZE +
      Hash.SIZE +
      UINT256.SIZE +
      UINT256.SIZE +
      1 +
      UINT256.SIZE +
      UINT256.SIZE
    )
  }

  static deserialize(arr: Uint8Array) {
    const items = u8aSplit(arr, [
      PublicKey.SIZE_UNCOMPRESSED,
      PublicKey.SIZE_UNCOMPRESSED,
      Balance.SIZE,
      Hash.SIZE,
      UINT256.SIZE,
      UINT256.SIZE,
      1,
      UINT256.SIZE,
      UINT256.SIZE
    ])

    return new ChannelEntry(
      PublicKey.deserialize(items[0]),
      PublicKey.deserialize(items[1]),
      Balance.deserialize(items[2]),
      Hash.deserialize(items[3]),
      UINT256.deserialize(items[4]),
      UINT256.deserialize(items[5]),
      u8aToChannelStatus(items[6]),
      UINT256.deserialize(items[7]),
      UINT256.deserialize(items[8])
    )
  }

  static async fromSCEvent(
    event: ChannelUpdateEvent,
    keyFor: (a: Address) => Promise<PublicKey>
  ): Promise<ChannelEntry> {
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
      [this.source.serializeUncompressed(), PublicKey.SIZE_UNCOMPRESSED],
      [this.destination.serializeUncompressed(), PublicKey.SIZE_UNCOMPRESSED],
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
      `ChannelEntry   (${chalk.green(this.getId().toHex())}):\n` +
      `  source:       ${chalk.yellow(this.source.toAddress())} (${this.source.toString()})\n` +
      `  destination:  ${chalk.yellow(this.destination.toAddress())} (${this.destination.toString()})\n` +
      `  balance:      ${this.balance.toFormattedString()}\n` +
      `  commitment:   ${this.commitment.toHex()}\n` +
      `  ticketEpoch:  ${this.ticketEpoch.toBN().toString(10)}\n` +
      `  ticketIndex:  ${this.ticketIndex.toBN().toString(10)}\n` +
      `  status:       ${chalk.green(channelStatusToString(this.status))}\n` +
      `  channelEpoch: ${this.channelEpoch.toBN().toString(10)}\n` +
      `  closureTime:  ${this.closureTime.toBN().toString(10)}\n`
    )
  }

  public getId() {
    return generateChannelId(this.source.toAddress(), this.destination.toAddress())
  }

  /*
   * Calculates whether the channel has passed its required closure time
   * window.
   * @returns true if the time window passed, false if not
   */
  public closureTimePassed(): boolean {
    const nowInSeconds = Math.floor(Date.now() / 1000)
    const now = new BN(nowInSeconds)
    return !!this.closureTime && now.gt(this.closureTime.toBN())
  }

  /**
   * Computes the remaining time in seconds until the channel can be closed.
   * Outputs `0` if there is no waiting time, and `-1` if the
   * closure time of this channel is unknown.
   * @dev used to create more comprehensive debug logs
   */
  public getRemainingClosureTime(): BN {
    const nowInSeconds = Math.round(new Date().getTime() / 1000)
    const now = new BN(nowInSeconds)

    if (this.closureTime == undefined) {
      return new BN(-1)
    }

    return now.gt(this.closureTime.toBN()) ? new BN(0) : this.closureTime.toBN().sub(now)
  }

  public static createMock(): ChannelEntry {
    const pub = PublicKey.createMock()
    return new ChannelEntry(
      pub,
      pub,
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
