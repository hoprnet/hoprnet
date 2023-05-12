import { Address, Balance, BalanceType, ChannelStatus, Hash, PublicKey, U256 } from '@hoprnet/hopr-utils'
import { ChannelEntry, stringToU8a } from '@hoprnet/hopr-utils'
import { BigNumberish } from 'ethers'

export type IndexerSnapshot = { blockNumber: number; transactionIndex: number; logIndex: number }

/**
 * Compares the two snapshots provided.
 * @param snapA
 * @param snapB
 * @returns 0 if they're equal, negative if `a` goes up, positive if `b` goes up
 */
export function snapshotComparator(snapA: IndexerSnapshot, snapB: IndexerSnapshot): number {
  if (snapA.blockNumber != snapB.blockNumber) {
    return snapA.blockNumber - snapB.blockNumber
  } else if (snapA.transactionIndex != snapB.transactionIndex) {
    return snapA.transactionIndex - snapB.transactionIndex
  } else {
    return snapA.logIndex - snapB.logIndex
  }
}

/**
 * Compares blockNumber and onChainBlockNumber and returns `true`
 * if blockNumber is considered confirmed.
 * @returns boolean
 */
export function isConfirmedBlock(blockNumber: number, onChainBlockNumber: number, maxConfirmations: number): boolean {
  return blockNumber + maxConfirmations <= onChainBlockNumber
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

export async function channelEntryFromSCEvent(
  event: ChannelUpdateEvent,
  keyFor: (a: Address) => Promise<PublicKey>
): Promise<ChannelEntry> {
  const { source, destination, newState } = event.args

  return new ChannelEntry(
    await keyFor(Address.from_string(source)),
    await keyFor(Address.from_string(destination)),
    new Balance(newState.balance.toString(), BalanceType.HOPR),
    new Hash(stringToU8a(newState.commitment)),
    new U256(newState.ticketEpoch.toString()),
    new U256(newState.ticketIndex.toString()),
    numberToChannelStatus(newState.status),
    new U256(newState.channelEpoch.toString()),
    new U256(newState.closureTime.toString())
  )
}
