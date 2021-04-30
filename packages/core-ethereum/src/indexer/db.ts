import type { LevelUp } from 'levelup'
import BN from 'bn.js'
import { Hash, Address, AccountEntry, ChannelEntry, Snapshot } from '../types'
import { getFromDB } from '../utils'

const encoder = new TextEncoder()
const LATEST_BLOCK_NUMBER_KEY = encoder.encode('indexer-latestBlockNumber')
const LATEST_CONFIRMED_SNAPSHOT_KEY = encoder.encode('indexer-latestConfirmedSnapshot')
const createChannelKey = (channelId: Hash): Uint8Array => encoder.encode(`indexer-channel-${channelId.toHex()}`)
const createAccountKey = (address: Address): Uint8Array => encoder.encode(`indexer-account-${address.toHex()}`)

/**
 * Queries the database to find the latest known block number.
 * @param db
 * @returns promise that resolves to a number
 */
export const getLatestBlockNumber = async (db: LevelUp): Promise<number> => {
  const data = await getFromDB<Uint8Array>(db, LATEST_BLOCK_NUMBER_KEY)
  if (!data) return 0
  return new BN(data).toNumber()
}

/**
 * Updates the database with the latest known block number.
 * @param db
 * @param blockNumber
 */
export const updateLatestBlockNumber = async (db: LevelUp, blockNumber: BN): Promise<void> => {
  await db.put(Buffer.from(LATEST_BLOCK_NUMBER_KEY), blockNumber.toBuffer())
}

/**
 * Queries the database to find the latest confirmed snapshot.
 * @param db
 * @returns promise that resolves to a snapshot
 */
export const getLatestConfirmedSnapshot = async (db: LevelUp): Promise<Snapshot | undefined> => {
  const data = await getFromDB<Uint8Array>(db, LATEST_CONFIRMED_SNAPSHOT_KEY)
  if (!data || data.length != Snapshot.SIZE) {
    return undefined
  }
  return Snapshot.deserialize(data)
}

/**
 * Update latest confirmed snapshot.
 * @param db
 * @param snapshot
 */
export const updateLatestConfirmedSnapshot = async (db: LevelUp, snapshot: Snapshot): Promise<void> => {
  await db.put(Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY), Buffer.from(snapshot.serialize()))
}

/**
 * Queries the database to find the channel entry
 * @param db
 * @param address
 */
export const getChannel = async (db: LevelUp, channelId: Hash): Promise<ChannelEntry | undefined> => {
  const data = await getFromDB<Uint8Array>(db, createChannelKey(channelId))
  if (!data || data.length != ChannelEntry.SIZE) {
    return undefined
  }
  return ChannelEntry.deserialize(data)
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
        if (data == null || data.length != ChannelEntry.SIZE) return
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
 * @param db
 * @param address
 * @param channelEntry
 */
export const updateChannel = async (db: LevelUp, channelId: Hash, channel: ChannelEntry): Promise<void> => {
  await db.put(Buffer.from(createChannelKey(channelId)), Buffer.from(channel.serialize()))
}

/**
 * Queries the database to find an account
 * @param db
 * @param address
 * @returns Account
 */
export const getAccount = async (db: LevelUp, address: Address): Promise<AccountEntry | undefined> => {
  const data = await getFromDB<Uint8Array>(db, createAccountKey(address))
  if (!data || data.length != AccountEntry.SIZE) {
    return undefined
  }
  return AccountEntry.deserialize(data)
}

/**
 * Adds or updates the channel entry in the database.
 * Adds or updates latest confirmed snapshot.
 * @param db
 * @param partyA
 * @param partyB
 * @param channelEntry
 */
export const updateAccount = async (db: LevelUp, address: Address, account: AccountEntry): Promise<void> => {
  await db.put(Buffer.from(createAccountKey(address)), Buffer.from(account.serialize()))
}

export const getAccounts = async (db: LevelUp, filter?: (account: AccountEntry) => Promise<boolean>) => {
  const accounts: AccountEntry[] = []

  return new Promise<AccountEntry[]>((resolve, reject) => {
    db.createValueStream({
      keys: false,
      values: true
    })
      .on('data', async (data) => {
        if (data == null || data.length != ChannelEntry.SIZE) return
        const account = AccountEntry.deserialize(data)

        if (!filter || (await filter(account))) {
          accounts.push(account)
        }
      })
      .on('end', () => resolve(accounts))
      .on('error', reject)
  })
}
