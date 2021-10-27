import levelup from 'levelup'
import type { LevelUp } from 'levelup'
import leveldown from 'leveldown'
import MemDown from 'memdown'
import { existsSync, mkdirSync, rmSync } from 'fs'
import path from 'path'
import { debug } from './debug'
import { Hash, u8aConcat, Address, Intermediate, Ticket, generateChannelId } from '.'
import {
  AcknowledgedTicket,
  UnacknowledgedTicket,
  AccountEntry,
  ChannelEntry,
  Snapshot,
  PublicKey,
  Balance,
  HalfKeyChallenge,
  EthereumChallenge,
  UINT256
} from './types'
import BN from 'bn.js'
import { u8aEquals, u8aToNumber } from './u8a'

const log = debug(`hopr-core:db`)
const encoder = new TextEncoder()

const TICKET_PREFIX = encoder.encode('tickets-')
const SEPARATOR = encoder.encode(':')
const UNACKNOWLEDGED_TICKETS_PREFIX = u8aConcat(TICKET_PREFIX, encoder.encode('unacknowledged-'))
const ACKNOWLEDGED_TICKETS_PREFIX = u8aConcat(TICKET_PREFIX, encoder.encode('acknowledged-'))
export const unacknowledgedTicketKey = (halfKey: HalfKeyChallenge) => {
  return u8aConcat(UNACKNOWLEDGED_TICKETS_PREFIX, halfKey.serialize())
}
const acknowledgedTicketKey = (challenge: EthereumChallenge, channelEpoch: UINT256) => {
  return u8aConcat(ACKNOWLEDGED_TICKETS_PREFIX, channelEpoch.serialize(), SEPARATOR, challenge.serialize())
}
const PACKET_TAG_PREFIX: Uint8Array = encoder.encode('packets-tag-')
const LATEST_BLOCK_NUMBER_KEY = encoder.encode('indexer-latestBlockNumber')
const LATEST_CONFIRMED_SNAPSHOT_KEY = encoder.encode('indexer-latestConfirmedSnapshot')
const ACCOUNT_PREFIX = encoder.encode('indexer-account-')
const CHANNEL_PREFIX = encoder.encode('indexer-channel-')
const createChannelKey = (channelId: Hash): Uint8Array => u8aConcat(CHANNEL_PREFIX, encoder.encode(channelId.toHex()))
const createAccountKey = (address: Address): Uint8Array => u8aConcat(ACCOUNT_PREFIX, encoder.encode(address.toHex()))
const COMMITMENT_PREFIX = encoder.encode('commitment:')
const TICKET_INDEX_PREFIX = encoder.encode('ticketIndex:')
const CURRENT = encoder.encode('current')
const REDEEMED_TICKETS_COUNT = encoder.encode('statistics:redeemed:count')
const REDEEMED_TICKETS_VALUE = encoder.encode('statistics:redeemed:value')
const LOSING_TICKET_COUNT = encoder.encode('statistics:losing:count')
const PENDING_TICKETS_VALUE = (address: Address) =>
  u8aConcat(encoder.encode('statistics:pending:value:'), encoder.encode(address.toHex()))

export class HoprDB {
  private db: LevelUp

  constructor(private id: PublicKey, initialize: boolean, version: string, dbPath?: string, forceCreate?: boolean) {
    if (!dbPath) {
      dbPath = path.join(process.cwd(), 'db', version)
    }

    dbPath = path.resolve(dbPath)

    log('using db at ', dbPath)
    if (forceCreate) {
      log('force create - wipe old database and create a new')
      rmSync(dbPath, { recursive: true, force: true })
      mkdirSync(dbPath, { recursive: true })
    }
    if (!existsSync(dbPath)) {
      log('db does not exist, creating?:', initialize)
      if (initialize) {
        mkdirSync(dbPath, { recursive: true })
      } else {
        throw new Error('Database does not exist: ' + dbPath)
      }
    }
    this.db = levelup(leveldown(dbPath))
    log('namespacing db by pubkey: ', id.toAddress().toHex())
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
    return Uint8Array.from(await this.db.get(Buffer.from(this.keyOf(key))))
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

  private async getCoerced<T>(key: Uint8Array, coerce: (u: Uint8Array) => T) {
    let u8a = await this.get(key)
    return coerce(u8a)
  }

  private async getCoercedOrDefault<T>(key: Uint8Array, coerce: (u: Uint8Array) => T, defaultVal: T) {
    let u8a = await this.maybeGet(key)
    if (u8a === undefined) {
      return defaultVal
    }
    return coerce(u8a)
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

  private async increment(key: Uint8Array): Promise<number> {
    let val = await this.getCoercedOrDefault<number>(key, u8aToNumber, 0)
    await this.put(key, Uint8Array.of(val + 1))
    return val + 1
  }

  private async addBalance(key: Uint8Array, amount: Balance): Promise<void> {
    let val = await this.getCoercedOrDefault<Balance>(key, Balance.deserialize, Balance.ZERO())
    await this.put(key, val.add(amount).serialize())
  }

  private async subBalance(key: Uint8Array, amount: Balance): Promise<void> {
    let val = await this.getCoercedOrDefault<Balance>(key, Balance.deserialize, Balance.ZERO())
    await this.put(key, new Balance(val.toBN().sub(amount.toBN())).serialize())
  }

  /**
   * Get unacknowledged tickets.
   * @param filter optionally filter by signer
   * @returns an array of all unacknowledged tickets
   */
  public async getUnacknowledgedTickets(filter?: { signer: PublicKey }): Promise<UnacknowledgedTicket[]> {
    const filterFunc = (u: UnacknowledgedTicket): boolean => {
      // if signer provided doesn't match our ticket's signer dont add it to the list
      if (filter?.signer && u.signer.eq(filter.signer)) {
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

  public async getUnacknowledgedTicket(halfKeyChallenge: HalfKeyChallenge): Promise<UnacknowledgedTicket> {
    return UnacknowledgedTicket.deserialize(await this.get(unacknowledgedTicketKey(halfKeyChallenge)))
  }

  public async storeUnacknowledgedTicket(
    halfKeyChallenge: HalfKeyChallenge,
    unackTicket: UnacknowledgedTicket
  ): Promise<void> {
    await this.put(unacknowledgedTicketKey(halfKeyChallenge), unackTicket.serialize())
  }

  /**
   * Get acknowledged tickets
   * @param filter optionally filter by signer
   * @returns an array of all acknowledged tickets
   */
  public async getAcknowledgedTickets(filter?: { signer: PublicKey }): Promise<AcknowledgedTicket[]> {
    const filterFunc = (a: AcknowledgedTicket): boolean => {
      // if signer provided doesn't match our ticket's signer dont add it to the list
      if (filter?.signer && !a.signer.eq(filter.signer)) {
        return false
      }
      return true
    }

    return this.getAll<AcknowledgedTicket>(ACKNOWLEDGED_TICKETS_PREFIX, AcknowledgedTicket.deserialize, filterFunc)
  }

  /**
   * Delete acknowledged ticket in database
   * @param index Uint8Array
   */
  public async delAcknowledgedTicket(ack: AcknowledgedTicket, channelEpoch: UINT256): Promise<void> {
    await this.del(acknowledgedTicketKey(ack.ticket.challenge, channelEpoch))
  }

  public async replaceUnAckWithAck(halfKeyChallenge: HalfKeyChallenge, ackTicket: AcknowledgedTicket): Promise<void> {
    const unAcknowledgedDbKey = unacknowledgedTicketKey(halfKeyChallenge)
    const acknowledgedDbKey = acknowledgedTicketKey(ackTicket.ticket.challenge, ackTicket.ticket.channelEpoch)

    await this.db
      .batch()
      .del(Buffer.from(this.keyOf(unAcknowledgedDbKey)))
      .put(Buffer.from(this.keyOf(acknowledgedDbKey)), Buffer.from(ackTicket.serialize()))
      .write()
  }

  /**
   * Get tickets, both unacknowledged and acknowledged
   * @param node
   * @param filter optionally filter by signer
   * @returns an array of signed tickets
   */
  public async getTickets(filter?: { signer: PublicKey }): Promise<Ticket[]> {
    return Promise.all([this.getUnacknowledgedTickets(filter), this.getAcknowledgedTickets(filter)]).then(
      async ([unAcks, acks]) => {
        const unAckTickets = await Promise.all(unAcks.map((o) => o.ticket))
        const ackTickets = await Promise.all(acks.map((o) => o.ticket))
        return [...unAckTickets, ...ackTickets]
      }
    )
  }

  async checkAndSetPacketTag(packetTag: Uint8Array) {
    let present = await this.has(this.keyOf(PACKET_TAG_PREFIX, packetTag))

    if (!present) {
      await this.touch(this.keyOf(PACKET_TAG_PREFIX, packetTag))
    }

    return present
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
    return new Hash(await this.get(Uint8Array.from([...COMMITMENT_PREFIX, ...CURRENT, ...channelId.serialize()])))
  }

  setCurrentCommitment(channelId: Hash, commitment: Hash): Promise<void> {
    return this.put(
      Uint8Array.from([...COMMITMENT_PREFIX, ...CURRENT, ...channelId.serialize()]),
      commitment.serialize()
    )
  }

  async getCurrentTicketIndex(channelId: Hash): Promise<UINT256 | undefined> {
    return await this.getCoercedOrDefault(
      Uint8Array.from([...TICKET_INDEX_PREFIX, ...CURRENT, ...channelId.serialize()]),
      UINT256.deserialize,
      undefined
    )
  }

  setCurrentTicketIndex(channelId: Hash, ticketIndex: UINT256): Promise<void> {
    return this.put(
      Uint8Array.from([...TICKET_INDEX_PREFIX, ...CURRENT, ...channelId.serialize()]),
      ticketIndex.serialize()
    )
  }

  async getLatestBlockNumber(): Promise<number> {
    if (!(await this.has(LATEST_BLOCK_NUMBER_KEY))) return 0
    return new BN(await this.get(LATEST_BLOCK_NUMBER_KEY)).toNumber()
  }

  async updateLatestBlockNumber(blockNumber: BN): Promise<void> {
    await this.put(LATEST_BLOCK_NUMBER_KEY, blockNumber.toBuffer())
  }

  async getLatestConfirmedSnapshotOrUndefined(): Promise<Snapshot | undefined> {
    return await this.getCoercedOrDefault(LATEST_CONFIRMED_SNAPSHOT_KEY, Snapshot.deserialize, undefined)
  }

  async updateLatestConfirmedSnapshot(snapshot: Snapshot): Promise<void> {
    await this.put(LATEST_CONFIRMED_SNAPSHOT_KEY, snapshot.serialize())
  }

  async getChannel(channelId: Hash): Promise<ChannelEntry> {
    return await this.getCoerced(createChannelKey(channelId), ChannelEntry.deserialize)
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
    return data ? AccountEntry.deserialize(data) : undefined
  }

  async updateAccount(account: AccountEntry): Promise<void> {
    await this.put(createAccountKey(account.address), account.serialize())
  }

  async getAccounts(filter?: (account: AccountEntry) => boolean) {
    filter = filter || (() => true)
    return this.getAll<AccountEntry>(ACCOUNT_PREFIX, AccountEntry.deserialize, filter)
  }

  public async getRedeemedTicketsValue(): Promise<Balance> {
    return await this.getCoercedOrDefault(REDEEMED_TICKETS_VALUE, Balance.deserialize, Balance.ZERO())
  }
  public async getRedeemedTicketsCount(): Promise<number> {
    return this.getCoercedOrDefault(REDEEMED_TICKETS_COUNT, u8aToNumber, 0)
  }

  public async getPendingTicketCount(): Promise<number> {
    return (await this.getUnacknowledgedTickets()).length
  }

  public async getPendingBalanceTo(counterparty: Address): Promise<Balance> {
    return await this.getCoercedOrDefault(PENDING_TICKETS_VALUE(counterparty), Balance.deserialize, Balance.ZERO())
  }

  public async getLosingTicketCount(): Promise<number> {
    return this.getCoercedOrDefault(LOSING_TICKET_COUNT, u8aToNumber, 0)
  }

  public async markPending(ticket: Ticket) {
    await this.addBalance(PENDING_TICKETS_VALUE(ticket.counterparty), ticket.amount)
  }

  public async markRedeemeed(a: AcknowledgedTicket): Promise<void> {
    await this.increment(REDEEMED_TICKETS_COUNT)
    await this.delAcknowledgedTicket(a, a.ticket.channelEpoch)
    await this.addBalance(REDEEMED_TICKETS_VALUE, a.ticket.amount)
    await this.subBalance(PENDING_TICKETS_VALUE(a.ticket.counterparty), a.ticket.amount)
  }

  public async markLosing(t: UnacknowledgedTicket): Promise<void> {
    await this.increment(LOSING_TICKET_COUNT)
    await this.del(unacknowledgedTicketKey(t.getChallenge()))
    // sub pending_tickets_value
  }

  static createMock(id?: PublicKey): HoprDB {
    const mock: HoprDB = {
      id: id ?? PublicKey.createMock(),
      db: new levelup(MemDown())
    } as any
    Object.setPrototypeOf(mock, HoprDB.prototype)

    return mock
  }

  public async getChannelX(src: PublicKey, dest: PublicKey): Promise<ChannelEntry> {
    return await this.getChannel(generateChannelId(src.toAddress(), dest.toAddress()))
  }

  public async getChannelTo(dest: PublicKey): Promise<ChannelEntry> {
    return await this.getChannel(generateChannelId(this.id.toAddress(), dest.toAddress()))
  }

  public async getChannelFrom(src: PublicKey): Promise<ChannelEntry> {
    return await this.getChannel(generateChannelId(src.toAddress(), this.id.toAddress()))
  }

  public async getChannelsFrom(address: Address) {
    return this.getChannels((channel) => {
      return address.eq(channel.source.toAddress())
    })
  }

  public async getChannelsTo(address: Address) {
    return this.getChannels((channel) => {
      return address.eq(channel.destination.toAddress())
    })
  }
}
