import levelup, { LevelUp } from 'levelup'
import leveldown from 'leveldown'
import MemDown from 'memdown'
import { existsSync, mkdirSync } from 'fs'
import path from 'path'
import Debug from 'debug'
import { Hash, u8aAdd, toU8a, u8aConcat, HalfKeyChallenge, Address, Intermediate } from '.'
import assert from 'assert'
import {
  Ticket,
  AcknowledgedTicket,
  UnacknowledgedTicket,
  AccountEntry,
  ChannelEntry,
  Snapshot,
  PublicKey
} from './types'
import BN from 'bn.js'
import { u8aEquals } from './u8a'

const log = Debug(`hopr-core:db`)

const encoder = new TextEncoder()
const TICKET_PREFIX: Uint8Array = encoder.encode('tickets-')
const PACKET_TAG_PREFIX: Uint8Array = encoder.encode('packets-tag-')
const ACKNOWLEDGED_TICKET_COUNTER = encoder.encode('tickets-acknowledgedCounter')
const UNACKNOWLEDGED_TICKETS_PREFIX = u8aConcat(TICKET_PREFIX, encoder.encode('unacknowledged-'))

const ACKNOWLEDGED_TICKET_INDEX_LENGTH = 8
const ACKNOWLEDGED_TICKET_PREFIX = u8aConcat(TICKET_PREFIX, encoder.encode('acknowledged-'))
const LATEST_BLOCK_NUMBER_KEY = encoder.encode('indexer-latestBlockNumber')
const LATEST_CONFIRMED_SNAPSHOT_KEY = encoder.encode('indexer-latestConfirmedSnapshot')
const ACCOUNT_PREFIX = encoder.encode('indexer-account-')
const CHANNEL_PREFIX = encoder.encode('indexer-channel-')
const createChannelKey = (channelId: Hash): Uint8Array => u8aConcat(CHANNEL_PREFIX, encoder.encode(channelId.toHex()))
const createAccountKey = (address: Address): Uint8Array => u8aConcat(ACCOUNT_PREFIX, encoder.encode(address.toHex()))
const COMMITMENT_PREFIX = encoder.encode('commitment:')
const CURRENT = encoder.encode('current')

function keyAcknowledgedTickets(index: Uint8Array): Uint8Array {
  assert(index.length == ACKNOWLEDGED_TICKET_INDEX_LENGTH)
  return u8aConcat(ACKNOWLEDGED_TICKET_PREFIX, index)
}

export function UnAcknowledgedTickets(encodedAckChallenge: HalfKeyChallenge): Uint8Array {
  return u8aConcat(UNACKNOWLEDGED_TICKETS_PREFIX, encodedAckChallenge.serialize())
}

export class HoprDB {
  private db: LevelUp

  constructor(private id: Address, initialize: boolean, version: string, dbPath?: string) {
    if (!dbPath) {
      dbPath = path.join(process.cwd(), 'db', version)
    }

    dbPath = path.resolve(dbPath)

    log('using db at ', dbPath)
    if (!existsSync(dbPath)) {
      log('db does not exist, creating?:', initialize)
      if (initialize) {
        mkdirSync(dbPath, { recursive: true })
      } else {
        throw new Error('Database does not exist: ' + dbPath)
      }
    }
    this.db = levelup(leveldown(dbPath))
    log('namespacing db by pubkey: ', id.toHex())
  }

  private keyOf(...segments: Uint8Array[]): Uint8Array {
    return u8aConcat(encoder.encode(this.id.toHex()), ...segments)
  }

  private async has(key: Uint8Array): Promise<boolean> {
    try {
      await this.db.get(Buffer.from(this.keyOf(key)))

      return true
    } catch (err) {
      if (err.type === 'NotFoundError' || err.notFound) {
        return false
      } else {
        throw err
      }
    }
  }

  private async put(key: Uint8Array, value: Uint8Array): Promise<void> {
    await this.db.put(Buffer.from(this.keyOf(key)), Buffer.from(value))
  }

  private async touch(key: Uint8Array): Promise<void> {
    return await this.put(key, new Uint8Array())
  }

  private async get(key: Uint8Array): Promise<Uint8Array> {
    return await this.db.get(Buffer.from(this.keyOf(key)))
  }

  private async maybeGet(key: Uint8Array): Promise<Uint8Array | undefined> {
    try {
      return await this.get(key)
    } catch (err) {
      if (err.type === 'NotFoundError' || err.notFound) {
        return undefined
      }
      throw err
    }
  }

  private async getAll<T>(
    prefix: Uint8Array,
    deserialize: (u: Uint8Array) => T,
    filter: (o: T) => boolean
  ): Promise<T[]> {
    const res: T[] = []
    const prefixKeyed = this.keyOf(prefix)
    return new Promise<T[]>((resolve, reject) => {
      this.db
        .createReadStream()
        .on('error', reject)
        .on('data', async ({ key, value }: { key: Buffer; value: Buffer }) => {
          if (!u8aEquals(key.subarray(0, prefixKeyed.length), prefixKeyed)) {
            return
          }
          const obj = deserialize(Uint8Array.from(value))
          if (filter(obj)) {
            res.push(obj)
          }
        })
        .on('end', () => resolve(res))
    })
  }

  private async del(key: Uint8Array): Promise<void> {
    await this.db.del(Buffer.from(this.keyOf(key)))
  }

  /**
   * Get all unacknowledged tickets
   * @param filter optionally filter by signer
   * @returns an array of all unacknowledged tickets
   */
  public async getUnacknowledgedTickets(filter?: { signer: Uint8Array }): Promise<UnacknowledgedTicket[]> {
    const filterFunc = (u: UnacknowledgedTicket): boolean => {
      // if signer provided doesn't match our ticket's signer dont add it to the list
      if (filter?.signer && u.ticket.verify(new PublicKey(filter.signer))) {
        return false
      }
      return true
    }
    return this.getAll<UnacknowledgedTicket>(
      UNACKNOWLEDGED_TICKETS_PREFIX,
      UnacknowledgedTicket.deserialize,
      filterFunc
    )
  }

  /**
   * Delete unacknowledged tickets
   * @param filter optionally filter by signer
   */
  async deleteUnacknowledgedTickets(filter?: { signer: Uint8Array }): Promise<void> {
    const tickets = await this.getUnacknowledgedTickets(filter)

    await this.db.batch(
      await Promise.all(
        tickets.map<any>(async (ticket) => {
          return {
            type: 'del',
            key: Buffer.from(this.keyOf(UnAcknowledgedTickets(ticket.ownKey.toChallenge())))
          }
        })
      )
    )
  }

  /**
   * Get all acknowledged tickets
   * @param filter optionally filter by signer
   * @returns an array of all acknowledged tickets
   */
  async getAcknowledgedTickets(filter?: { signer: Uint8Array }): Promise<AcknowledgedTicket[]> {
    const filterFunc = (a: AcknowledgedTicket): boolean => {
      // if signer provided doesn't match our ticket's signer dont add it to the list
      if (filter?.signer && a.ticket.verify(new PublicKey(filter.signer))) {
        return false
      }
      return true
    }
    return this.getAll<AcknowledgedTicket>(ACKNOWLEDGED_TICKET_PREFIX, AcknowledgedTicket.deserialize, filterFunc)
  }

  /**
   * Delete acknowledged tickets
   * @param filter optionally filter by signer
   */
  async deleteAcknowledgements(filter?: { signer: Uint8Array }): Promise<void> {
    const acks = await this.getAcknowledgedTickets(filter)
    await this.db.batch(
      await Promise.all(
        acks.map<any>(async (ack) => {
          return {
            type: 'del',
            key: Buffer.from(this.keyOf(keyAcknowledgedTickets(ack.ticket.challenge.serialize())))
          }
        })
      )
    )
  }

  /**
   * Update acknowledged ticket in database
   * @param ackTicket Uint8Array
   * @param index Uint8Array
   */
  async updateAcknowledgement(ackTicket: AcknowledgedTicket, index: Uint8Array): Promise<void> {
    await this.put(keyAcknowledgedTickets(index), ackTicket.serialize())
  }

  /**
   * Delete acknowledged ticket in database
   * @param index Uint8Array
   */
  async deleteAcknowledgement(acknowledgement: AcknowledgedTicket): Promise<void> {
    await this.del(keyAcknowledgedTickets(acknowledgement.ticket.challenge.serialize()))
  }

  /**
   * Get signed tickets, both unacknowledged and acknowledged
   * @param node
   * @param filter optionally filter by signer
   * @returns an array of signed tickets
   */
  async getTickets(filter?: { signer: Uint8Array }): Promise<Ticket[]> {
    return Promise.all([this.getUnacknowledgedTickets(filter), this.getAcknowledgedTickets(filter)]).then(
      async ([unAcks, acks]) => {
        const unAckTickets = await Promise.all(unAcks.map((o) => o.ticket))
        const ackTickets = await Promise.all(acks.map((o) => o.ticket))
        return [...unAckTickets, ...ackTickets]
      }
    )
  }

  /**
   * Get signed tickets, both unacknowledged and acknowledged
   * @param node
   * @param filter optionally filter by signer
   * @returns an array of signed tickets
   */
  async deleteTickets(filter?: { signer: Uint8Array }): Promise<void> {
    await Promise.all([this.deleteUnacknowledgedTickets(filter), this.deleteAcknowledgements(filter)])
  }

  async storeUnacknowledgedTickets(challenge: HalfKeyChallenge, unacknowledged: UnacknowledgedTicket) {
    await this.put(UnAcknowledgedTickets(challenge), unacknowledged.serialize())
  }

  async checkAndSetPacketTag(packetTag: Uint8Array) {
    let present = await this.has(this.keyOf(PACKET_TAG_PREFIX, packetTag))

    if (!present) {
      await this.touch(this.keyOf(PACKET_TAG_PREFIX, packetTag))
    }

    return present
  }

  async getUnacknowledgedTicketsByKey(challenge: HalfKeyChallenge): Promise<UnacknowledgedTicket | undefined> {
    const unAcknowledgedDbKey = UnAcknowledgedTickets(challenge)

    try {
      const buff = await this.get(unAcknowledgedDbKey)
      if (buff.length === 0) {
        return undefined
      }
      return UnacknowledgedTicket.deserialize(Uint8Array.from(buff))
    } catch (err) {
      if (err.notFound) {
        return undefined
      }
      throw err
    }
  }

  async deleteTicket(challenge: HalfKeyChallenge) {
    await this.del(UnAcknowledgedTickets(challenge))
  }

  async replaceTicketWithAcknowledgement(keyHalfChallenge: HalfKeyChallenge, acknowledgment: AcknowledgedTicket) {
    const ticketCounter = await this.getTicketCounter()
    const unAcknowledgedDbKey = UnAcknowledgedTickets(keyHalfChallenge)
    const acknowledgedDbKey = keyAcknowledgedTickets(ticketCounter)
    try {
      await this.db
        .batch()
        .del(Buffer.from(this.keyOf(unAcknowledgedDbKey)))
        .put(Buffer.from(this.keyOf(acknowledgedDbKey)), Buffer.from(acknowledgment.serialize()))
        .put(Buffer.from(this.keyOf(ACKNOWLEDGED_TICKET_COUNTER)), Buffer.from(ticketCounter))
        .write()
    } catch (err) {
      log(`ERROR: Error while writing to database. Error was ${err.message}.`)
    }
  }

  private async getTicketCounter(): Promise<Uint8Array> {
    try {
      let tmpTicketCounter = await this.get(ACKNOWLEDGED_TICKET_COUNTER)
      return u8aAdd(true, tmpTicketCounter, toU8a(1, ACKNOWLEDGED_TICKET_INDEX_LENGTH))
    } catch (err) {
      // Set ticketCounter to initial value
      return toU8a(0, ACKNOWLEDGED_TICKET_INDEX_LENGTH)
    }
  }

  async storeUnacknowledgedTicket(challenge: HalfKeyChallenge) {
    const unAcknowledgedDBKey = UnAcknowledgedTickets(challenge)
    await this.touch(unAcknowledgedDBKey)
    return unAcknowledgedDBKey
  }

  public close() {
    return this.db.close()
  }

  async storeHashIntermediaries(channelId: Hash, intermediates: Intermediate[]): Promise<void> {
    let dbBatch = this.db.batch()
    const keyFor = (iteration: number) =>
      this.keyOf(u8aConcat(COMMITMENT_PREFIX, channelId.serialize(), Uint8Array.of(iteration)))
    for (const intermediate of intermediates) {
      dbBatch = dbBatch.put(Buffer.from(keyFor(intermediate.iteration)), Buffer.from(intermediate.preImage))
    }
    await dbBatch.write()
  }

  async getCommitment(channelId: Hash, iteration: number) {
    return await this.maybeGet(u8aConcat(COMMITMENT_PREFIX, channelId.serialize(), Uint8Array.of(iteration)))
  }

  async getCurrentCommitment(channelId: Hash): Promise<Hash> {
    return new Hash(Uint8Array.from(await this.get(u8aConcat(COMMITMENT_PREFIX, CURRENT, channelId.serialize()))))
  }

  async setCurrentCommitment(channelId: Hash, commitment: Hash) {
    return this.put(u8aConcat(COMMITMENT_PREFIX, CURRENT, channelId.serialize()), commitment.serialize())
  }

  async getLatestBlockNumber(): Promise<number> {
    if (!(await this.has(LATEST_BLOCK_NUMBER_KEY))) return 0
    return new BN(await this.get(LATEST_BLOCK_NUMBER_KEY)).toNumber()
  }

  async updateLatestBlockNumber(blockNumber: BN): Promise<void> {
    await this.put(LATEST_BLOCK_NUMBER_KEY, blockNumber.toBuffer())
  }

  async getLatestConfirmedSnapshot(): Promise<Snapshot | undefined> {
    const data = await this.maybeGet(LATEST_CONFIRMED_SNAPSHOT_KEY)
    return data ? Snapshot.deserialize(Uint8Array.from(data)) : undefined
  }

  async updateLatestConfirmedSnapshot(snapshot: Snapshot): Promise<void> {
    await this.put(LATEST_CONFIRMED_SNAPSHOT_KEY, snapshot.serialize())
  }

  async getChannel(channelId: Hash): Promise<ChannelEntry | undefined> {
    const data = await this.maybeGet(createChannelKey(channelId))
    return data ? ChannelEntry.deserialize(Uint8Array.from(data)) : undefined
  }

  async getChannels(filter?: (channel: ChannelEntry) => boolean): Promise<ChannelEntry[]> {
    filter = filter || (() => true)
    return this.getAll<ChannelEntry>(CHANNEL_PREFIX, ChannelEntry.deserialize, filter)
  }

  async updateChannel(channelId: Hash, channel: ChannelEntry): Promise<void> {
    await this.put(createChannelKey(channelId), channel.serialize())
  }

  async getAccount(address: Address): Promise<AccountEntry | undefined> {
    const data = await this.maybeGet(createAccountKey(address))
    return data ? AccountEntry.deserialize(Uint8Array.from(data)) : undefined
  }

  async updateAccount(account: AccountEntry): Promise<void> {
    await this.put(createAccountKey(account.address), account.serialize())
  }

  async getAccounts(filter?: (account: AccountEntry) => boolean) {
    filter = filter || (() => true)
    return this.getAll<AccountEntry>(ACCOUNT_PREFIX, AccountEntry.deserialize, filter)
  }

  static createMock(): HoprDB {
    const mock: HoprDB = {
      id: Address.createMock(),
      db: new levelup(MemDown())
    } as any
    Object.setPrototypeOf(mock, HoprDB.prototype)

    return mock
  }
}
