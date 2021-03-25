import type { LevelUp } from 'levelup'
import BN from 'bn.js'
import { u8aToNumber } from '@hoprnet/hopr-utils'
import { Hash, Address, AccountEntry, ChannelEntry, Snapshot } from '../types'

const encoder = new TextEncoder()
const LATEST_BLOCK_NUMBER_KEY = encoder.encode('indexer-latestBlockNumber')
const LATEST_CONFIRMED_SNAPSHOT_KEY = encoder.encode('indexer-latestConfirmedSnapshot')
const createChannelKey = (channelId: Hash): Uint8Array => encoder.encode(`indexer-channel-${channelId.toHex()}`)
const createAccountKey = (address: Address): Uint8Array => encoder.encode(`indexer-account-${address.toHex()}`)

/**
 * Queries the database to find the latest known block number.
 * @param connector
 * @returns promise that resolves to a number
 */
export const getLatestBlockNumber = async (db: LevelUp): Promise<number> => {
  try {
    return u8aToNumber(await db.get(Buffer.from(LATEST_BLOCK_NUMBER_KEY))) as number
  } catch (err) {
    if (err.notFound) {
      return 0
    }

    throw err
  }
}

/**
 * Updates the database with the latest known block number.
 * @param connector
 * @param blockNumber
 */
export const updateLatestBlockNumber = async (db: LevelUp, blockNumber: BN): Promise<void> => {
  await db.put(Buffer.from(LATEST_BLOCK_NUMBER_KEY), blockNumber.toBuffer())
}

/**
 * Queries the database to find the latest confirmed snapshot.
 * @param connector
 * @returns promise that resolves to a snapshot
 */
export const getLatestConfirmedSnapshot = async (db: LevelUp): Promise<Snapshot> => {
  try {
    const result = (await db.get(Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY))) as Uint8Array
    return new Snapshot({
      bytes: result,
      offset: result.byteOffset
    })
  } catch (err) {
    if (err.notFound) {
      return new Snapshot(undefined, {
        blockNumber: new BN(0),
        transactionIndex: new BN(0),
        logIndex: new BN(0)
      })
    }

    throw err
  }
}

/**
 * Queries the database to find the channel entry
 * @param connector
 * @param address
 */
export const getChannel = async (db: LevelUp, channelId: Hash): Promise<ChannelEntry | undefined> => {
  let channel: Uint8Array | undefined
  try {
    channel = (await db.get(Buffer.from(createChannelKey(channelId)))) as Uint8Array
  } catch (err) {
    if (err.notFound) {
      return undefined
    }

    throw err
  }

  if (channel == null || channel.length == 0) {
    return undefined
  }

  const channelEntry = ChannelEntry.deserialize(channel)

  return channelEntry
}

export const getChannels = async (
  db: LevelUp,
  filter?: (channel: ChannelEntry) => Promise<boolean>
): Promise<ChannelEntry[]> => {
  const channels: ChannelEntry[] = []

  return new Promise<ChannelEntry[]>((resolve, reject) => {
    db.createValueStream({
      keys: false,
      values: true
    })
      .on('data', async (data) => {
        const channel = ChannelEntry.deserialize(data)

        if (!filter || (await filter(channel))) {
          channels.push(channel)
        }
      })
      .on('end', () => resolve(channels))
      .on('error', reject)
  })
}

/**
 * Adds or updates the channel entry in the database.
 * Adds or updates latest confirmed snapshot.
 * @param connector
 * @param address
 * @param channelEntry
 */
export const updateChannel = async (db: LevelUp, channelId: Hash, channel: ChannelEntry): Promise<void> => {
  await db.put(Buffer.from(createChannelKey(channelId)), Buffer.from(channel.serialize()))
}

/**
 * Queries the database to find an account
 * @param connector
 * @param address
 * @returns Account
 */
export const getAccount = async (db: LevelUp, address: Address): Promise<AccountEntry | undefined> => {
  let account: Uint8Array | undefined
  try {
    account = (await db.get(Buffer.from(createAccountKey(address)))) as Uint8Array
  } catch (err) {
    if (err.notFound) {
      return undefined
    }

    throw err
  }

  if (account == null || account.length == 0) {
    return undefined
  }

  return AccountEntry.deserialize(account)
}

/**
 * Adds or updates the channel entry in the database.
 * Adds or updates latest confirmed snapshot.
 * @param connector
 * @param partyA
 * @param partyB
 * @param channelEntry
 */
export const updateAccount = async (db: LevelUp, address: Address, account: AccountEntry): Promise<void> => {
  await db.put(Buffer.from(createAccountKey(address)), Buffer.from(account.serialize()))
}
