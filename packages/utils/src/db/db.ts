import type { LevelUp } from 'levelup'
import levelup from 'levelup'
import leveldown from 'leveldown'
import MemDown from 'memdown'
import { stat, mkdir, rm } from 'fs/promises'
import { debug } from '../process'
import { Intermediate } from '../crypto'
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
  UINT256,
  Ticket,
  Address,
  Hash,
  generateChannelId
} from '../types'
import BN from 'bn.js'
import { u8aToNumber, u8aConcat, toU8a } from '../u8a'

const log = debug(`hopr-core:db`)

const encoder = new TextEncoder()
const decoder = new TextDecoder()

// key seperator:    `:`
// key query prefix: `-`

const ACCOUNT_PREFIX = encoder.encode('account-')
const CHANNEL_PREFIX = encoder.encode('channel-')
const COMMITMENT_PREFIX = encoder.encode('commitment-')
const CURRENT_COMMITMENT_PREFIX = encoder.encode('commitment:current-')
const TICKET_INDEX_PREFIX = encoder.encode('ticketIndex-')
const PENDING_TICKETS_COUNT = encoder.encode('statistics:pending:value-')
const ACKNOWLEDGED_TICKETS_PREFIX = encoder.encode('tickets:acknowledged-')
const PENDING_ACKNOWLEDGEMENTS_PREFIX = encoder.encode('tickets:pending-acknowledgement-')
const PACKET_TAG_PREFIX: Uint8Array = encoder.encode('packets:tag-')
const WHITELIST_REGISTRY_PREFIX: Uint8Array = encoder.encode('whitelist:registry:')
const WHITELIST_ELIGIBLE_PREFIX: Uint8Array = encoder.encode('whitelist:eligible:')
const WHITELIST_ENABLED_PREFIX: Uint8Array = encoder.encode('whitelist:enabled')

function createChannelKey(channelId: Hash): Uint8Array {
  return Uint8Array.from([...CHANNEL_PREFIX, ...channelId.serialize()])
}
function createAccountKey(address: Address): Uint8Array {
  return Uint8Array.from([...ACCOUNT_PREFIX, ...address.serialize()])
}
function createCommitmentKey(channelId: Hash, iteration: number) {
  return Uint8Array.from([...COMMITMENT_PREFIX, ...channelId.serialize(), ...toU8a(iteration, 4)])
}
function createCurrentCommitmentKey(channelId: Hash) {
  return Uint8Array.from([...CURRENT_COMMITMENT_PREFIX, ...channelId.serialize()])
}
function createCurrentTicketIndexKey(channelId: Hash) {
  return Uint8Array.from([...TICKET_INDEX_PREFIX, ...channelId.serialize()])
}
function createPendingTicketsCountKey(address: Address) {
  return Uint8Array.from([...PENDING_TICKETS_COUNT, ...address.serialize()])
}
function createAcknowledgedTicketKey(challenge: EthereumChallenge, channelEpoch: UINT256) {
  return Uint8Array.from([...ACKNOWLEDGED_TICKETS_PREFIX, ...channelEpoch.serialize(), ...challenge.serialize()])
}
function createPendingAcknowledgement(halfKey: HalfKeyChallenge) {
  return Uint8Array.from([...PENDING_ACKNOWLEDGEMENTS_PREFIX, ...halfKey.serialize()])
}
function createPacketTagKey(tag: Uint8Array) {
  return Uint8Array.from([...PACKET_TAG_PREFIX, ...tag])
}
function createWhitelistRegistryKey(publicKey: PublicKey) {
  return Uint8Array.from([...WHITELIST_REGISTRY_PREFIX, ...publicKey.serializeUncompressed()])
}
function createWhitelistEligibleKey(address: Address) {
  return Uint8Array.from([...WHITELIST_ELIGIBLE_PREFIX, ...address.serialize()])
}

const LATEST_BLOCK_NUMBER_KEY = encoder.encode('latestBlockNumber')
const LATEST_CONFIRMED_SNAPSHOT_KEY = encoder.encode('latestConfirmedSnapshot')
const REDEEMED_TICKETS_COUNT = encoder.encode('statistics:redeemed:count')
const REDEEMED_TICKETS_VALUE = encoder.encode('statistics:redeemed:value')
const LOSING_TICKET_COUNT = encoder.encode('statistics:losing:count')
const NEGLECTED_TICKET_COUNT = encoder.encode('statistics:neglected:count')
const REJECTED_TICKETS_COUNT = encoder.encode('statistics:rejected:count')
const REJECTED_TICKETS_VALUE = encoder.encode('statistics:rejected:value')
const ENVIRONMENT_KEY = encoder.encode('environment_id')
const HOPR_BALANCE_KEY = encoder.encode('hopr-balance')

enum PendingAcknowledgementPrefix {
  Relayer = 0,
  MessageSender = 1
}

export type WaitingAsSender = {
  isMessageSender: true
}

export type WaitingAsRelayer = {
  isMessageSender: false
  ticket: UnacknowledgedTicket
}

export type PendingAckowledgement = WaitingAsSender | WaitingAsRelayer

function serializePendingAcknowledgement(isMessageSender: boolean, unackTicket?: UnacknowledgedTicket) {
  if (isMessageSender) {
    return Uint8Array.from([PendingAcknowledgementPrefix.MessageSender])
  } else {
    return Uint8Array.from([PendingAcknowledgementPrefix.Relayer, ...unackTicket.serialize()])
  }
}

function deserializePendingAcknowledgement(data: Uint8Array): PendingAckowledgement {
  switch (data[0] as PendingAcknowledgementPrefix) {
    case PendingAcknowledgementPrefix.MessageSender:
      return {
        isMessageSender: true
      }
    case PendingAcknowledgementPrefix.Relayer:
      return {
        isMessageSender: false,
        ticket: UnacknowledgedTicket.deserialize(data.slice(1))
      }
  }
}

export class HoprDB {
  private db: LevelUp

  constructor(private id: PublicKey) {}

  async init(initialize: boolean, dbPath: string, forceCreate: boolean = false, environmentId: string) {
    let setEnvironment = false

    log(`using db at ${dbPath}`)
    if (forceCreate) {
      log('force create - wipe old database and create a new')
      await rm(dbPath, { recursive: true, force: true })
      await mkdir(dbPath, { recursive: true })
      setEnvironment = true
    }

    let exists = false

    try {
      exists = !(await stat(dbPath)).isDirectory()
    } catch (err: any) {
      if (err.code === 'ENOENT') {
        exists = false
      } else {
        // Unexpected error, therefore throw it
        throw err
      }
    }

    if (!exists) {
      log('db directory does not exist, creating?:', initialize)
      if (initialize) {
        await mkdir(dbPath, { recursive: true })
        setEnvironment = true
      } else {
        throw new Error('Database does not exist: ' + dbPath)
      }
    }
    this.db = levelup(leveldown(dbPath))

    // Fully initialize database
    await this.db.open()

    log(`namespacing db by native address: ${this.id.toCompressedPubKeyHex()}`)
    if (setEnvironment) {
      log(`setting environment id ${environmentId} to db`)
      await this.setEnvironmentId(environmentId)
    } else {
      const hasEnvironmentKey = await this.verifyEnvironmentId(environmentId)

      if (!hasEnvironmentKey) {
        const storedId = await this.getEnvironmentId()

        throw new Error(`invalid db environment id: ${storedId} (expected: ${environmentId})`)
      }
    }
  }

  private keyOf(...segments: Uint8Array[]): Uint8Array {
    return u8aConcat(this.id.serializeUncompressed().slice(1), ...segments)
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

  protected async put(key: Uint8Array, value: Uint8Array): Promise<void> {
    await this.db.put(Buffer.from(this.keyOf(key)), Buffer.from(value))
  }

  private async touch(key: Uint8Array): Promise<void> {
    return await this.put(key, new Uint8Array())
  }

  protected async get(key: Uint8Array): Promise<Uint8Array> {
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

  /**
   * Gets a elements from the database of a kind.
   * Optionally applies `filter`then `map` then `sort` to the result.
   * @param range.prefix key prefix, such as `channels-`
   * @param range.suffixLength length of the appended identifier to distinguish elements
   * @param deserialize function to parse serialized objects
   * @param filter [optional] filter deserialized objects
   * @param map [optional] transform deserialized and filtered objects
   * @param sorter [optional] sort deserialized, filtered and transformed objects
   * @returns a Promises that resolves with the found elements
   */
  protected async getAll<Element, TransformedElement = Element>(
    range: {
      prefix: Uint8Array
      suffixLength: number
    },
    deserialize: (u: Uint8Array) => Element,
    filter?: ((o: Element) => boolean) | undefined,
    map?: (i: Element) => TransformedElement,
    sorter?: (e1: TransformedElement, e2: TransformedElement) => number
  ): Promise<TransformedElement[]> {
    const firstPrefixed = this.keyOf(range.prefix, new Uint8Array(range.suffixLength).fill(0x00))
    const lastPrefixed = this.keyOf(range.prefix, new Uint8Array(range.suffixLength).fill(0xff))

    const results: TransformedElement[] = []

    // @TODO fix types in @types/levelup package
    for await (const [_key, chunk] of this.db.iterator({
      gte: Buffer.from(firstPrefixed),
      lte: Buffer.from(lastPrefixed),
      keys: false
    }) as any) {
      const obj: Element = deserialize(Uint8Array.from(chunk))

      if (!filter || filter(obj)) {
        if (map) {
          results.push(map(obj))
        } else {
          results.push(obj as unknown as TransformedElement)
        }
      }
    }

    if (sorter) {
      // sort in-place
      results.sort(sorter)
    }

    return results
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
    let val = await this.getCoercedOrDefault<Balance>(key, Balance.deserialize, Balance.ZERO)
    await this.put(key, val.add(amount).serialize())
  }

  private async subBalance(key: Uint8Array, amount: Balance): Promise<void> {
    let val = await this.getCoercedOrDefault<Balance>(key, Balance.deserialize, Balance.ZERO)
    await this.put(key, new Balance(val.toBN().sub(amount.toBN())).serialize())
  }

  /**
   * Get unacknowledged tickets.
   * @param filter optionally filter by signer
   * @returns an array of all unacknowledged tickets
   */
  public async getUnacknowledgedTickets(filter?: { signer: PublicKey }): Promise<UnacknowledgedTicket[]> {
    const filterFunc = (pending: PendingAckowledgement): boolean => {
      if (pending.isMessageSender == true) {
        return false
      }

      // if signer provided doesn't match our ticket's signer dont add it to the list
      if (filter?.signer && pending.ticket.signer.eq(filter.signer)) {
        return false
      }
      return true
    }

    return await this.getAll<PendingAckowledgement, UnacknowledgedTicket>(
      {
        prefix: PENDING_ACKNOWLEDGEMENTS_PREFIX,
        suffixLength: HalfKeyChallenge.SIZE
      },
      deserializePendingAcknowledgement,
      filterFunc,
      (pending: WaitingAsRelayer) => pending.ticket
    )
  }

  public async getPendingAcknowledgement(halfKeyChallenge: HalfKeyChallenge): Promise<PendingAckowledgement> {
    return await this.getCoerced(createPendingAcknowledgement(halfKeyChallenge), deserializePendingAcknowledgement)
  }

  public async storePendingAcknowledgement(halfKeyChallenge: HalfKeyChallenge, isMessageSender: true): Promise<void>
  public async storePendingAcknowledgement(
    halfKeyChallenge: HalfKeyChallenge,
    isMessageSender: false,
    unackTicket: UnacknowledgedTicket
  ): Promise<void>

  public async storePendingAcknowledgement(
    halfKeyChallenge: HalfKeyChallenge,
    isMessageSender: boolean,
    unackTicket?: UnacknowledgedTicket
  ): Promise<void> {
    await this.put(
      createPendingAcknowledgement(halfKeyChallenge),
      serializePendingAcknowledgement(isMessageSender, unackTicket)
    )
  }

  /**
   * Get acknowledged tickets sorted by ticket index in ascending order.
   * @param filter optionally filter by signer
   * @returns an array of all acknowledged tickets
   */
  public async getAcknowledgedTickets(filter?: {
    signer?: PublicKey
    channel?: ChannelEntry
  }): Promise<AcknowledgedTicket[]> {
    const filterFunc = (a: AcknowledgedTicket): boolean => {
      // if signer provided doesn't match our ticket's signer dont add it to the list
      if (filter?.signer && !a.signer.eq(filter.signer)) {
        return false
      }

      if (
        filter?.channel &&
        (!a.signer.eq(filter.channel.source) ||
          !filter.channel.destination.eq(this.id) ||
          !a.ticket.channelEpoch.eq(filter.channel.channelEpoch))
      ) {
        return false
      }

      return true
    }
    // sort in ascending order by ticket index: 1,2,3,4,...
    const sortFunc = (t1: AcknowledgedTicket, t2: AcknowledgedTicket): number => t1.ticket.index.cmp(t2.ticket.index)

    return this.getAll<AcknowledgedTicket>(
      {
        prefix: ACKNOWLEDGED_TICKETS_PREFIX,
        suffixLength: EthereumChallenge.SIZE
      },
      AcknowledgedTicket.deserialize,
      filterFunc,
      undefined,
      sortFunc
    )
  }

  public async deleteAcknowledgedTicketsFromChannel(channel: ChannelEntry): Promise<void> {
    const tickets = await this.getAcknowledgedTickets({ signer: channel.source })

    let neglectedTickets = await this.getCoercedOrDefault<number>(NEGLECTED_TICKET_COUNT, u8aToNumber, 0)

    const batch = this.db.batch()

    for (const ack of tickets) {
      batch.del(Buffer.from(this.keyOf(createAcknowledgedTicketKey(ack.ticket.challenge, ack.ticket.channelEpoch))))
    }

    batch.put(Buffer.from(this.keyOf(NEGLECTED_TICKET_COUNT)), Uint8Array.of(neglectedTickets + 1))

    await batch.write()
  }

  /**
   * Delete acknowledged ticket in database
   * @param index Uint8Array
   */
  public async delAcknowledgedTicket(ack: AcknowledgedTicket): Promise<void> {
    await this.del(createAcknowledgedTicketKey(ack.ticket.challenge, ack.ticket.channelEpoch))
  }

  public async replaceUnAckWithAck(halfKeyChallenge: HalfKeyChallenge, ackTicket: AcknowledgedTicket): Promise<void> {
    const unAcknowledgedDbKey = createPendingAcknowledgement(halfKeyChallenge)
    const acknowledgedDbKey = createAcknowledgedTicketKey(ackTicket.ticket.challenge, ackTicket.ticket.channelEpoch)

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
    const [unAcks, acks] = await Promise.all([
      this.getUnacknowledgedTickets(filter),
      this.getAcknowledgedTickets(filter)
    ])

    const unAckTickets = unAcks.map((o: UnacknowledgedTicket) => o.ticket)
    const ackTickets = acks.map((o: AcknowledgedTicket) => o.ticket)
    return [...unAckTickets, ...ackTickets]
  }

  /**
   * Checks whether the given packet tag is present in the database.
   * If not, sets the packet tag and return false, otherwise return
   * true.
   * @param packetTag packet tag to check for
   * @returns a Promise that resolves to true if packet tag is present in db
   */
  async checkAndSetPacketTag(packetTag: Uint8Array) {
    let present = await this.has(createPacketTagKey(packetTag))

    if (!present) {
      await this.touch(createPacketTagKey(packetTag))
    }

    return present
  }

  public async close() {
    log('Closing database')
    return await this.db.close()
  }

  async storeHashIntermediaries(channelId: Hash, intermediates: Intermediate[]): Promise<void> {
    let dbBatch = this.db.batch()

    for (const intermediate of intermediates) {
      dbBatch = dbBatch.put(
        Buffer.from(this.keyOf(createCommitmentKey(channelId, intermediate.iteration))),
        Buffer.from(intermediate.preImage)
      )
    }
    await dbBatch.write()
  }

  async getCommitment(channelId: Hash, iteration: number) {
    return await this.maybeGet(createCommitmentKey(channelId, iteration))
  }

  async getCurrentCommitment(channelId: Hash): Promise<Hash> {
    return new Hash(await this.get(createCurrentCommitmentKey(channelId)))
  }

  async setCurrentCommitment(channelId: Hash, commitment: Hash): Promise<void> {
    return this.put(createCurrentCommitmentKey(channelId), commitment.serialize())
  }

  async getCurrentTicketIndex(channelId: Hash): Promise<UINT256 | undefined> {
    return await this.getCoercedOrDefault(createCurrentTicketIndexKey(channelId), UINT256.deserialize, undefined)
  }

  setCurrentTicketIndex(channelId: Hash, ticketIndex: UINT256): Promise<void> {
    return this.put(createCurrentTicketIndexKey(channelId), ticketIndex.serialize())
  }

  async getLatestBlockNumber(): Promise<number> {
    if (!(await this.has(LATEST_BLOCK_NUMBER_KEY))) {
      return 0
    }
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
    return this.getAll<ChannelEntry>(
      {
        prefix: CHANNEL_PREFIX,
        suffixLength: Hash.SIZE
      },
      ChannelEntry.deserialize,
      filter
    )
  }

  async updateChannelAndSnapshot(channelId: Hash, channel: ChannelEntry, snapshot: Snapshot): Promise<void> {
    await this.db
      .batch()
      .put(Buffer.from(this.keyOf(createChannelKey(channelId))), Buffer.from(channel.serialize()))
      .put(Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY), Buffer.from(snapshot.serialize()))
      .write()
  }

  async getAccount(address: Address): Promise<AccountEntry | undefined> {
    return await this.getCoercedOrDefault(createAccountKey(address), AccountEntry.deserialize, undefined)
  }

  async updateAccountAndSnapshot(account: AccountEntry, snapshot: Snapshot): Promise<void> {
    await this.db
      .batch()
      .put(Buffer.from(this.keyOf(createAccountKey(account.getAddress()))), Buffer.from(account.serialize()))
      .put(Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY), Buffer.from(snapshot.serialize()))
      .write()
  }

  async getAccounts(filter?: (account: AccountEntry) => boolean) {
    return this.getAll<AccountEntry>(
      {
        prefix: ACCOUNT_PREFIX,
        suffixLength: Address.SIZE
      },
      AccountEntry.deserialize,
      filter
    )
  }

  public async getRedeemedTicketsValue(): Promise<Balance> {
    return await this.getCoercedOrDefault(REDEEMED_TICKETS_VALUE, Balance.deserialize, Balance.ZERO)
  }
  public async getRedeemedTicketsCount(): Promise<number> {
    return this.getCoercedOrDefault(REDEEMED_TICKETS_COUNT, u8aToNumber, 0)
  }

  public async getNeglectedTicketsCount(): Promise<number> {
    return this.getCoercedOrDefault(NEGLECTED_TICKET_COUNT, u8aToNumber, 0)
  }

  public async getPendingTicketCount(): Promise<number> {
    return (await this.getUnacknowledgedTickets()).length
  }

  public async getPendingBalanceTo(counterparty: Address): Promise<Balance> {
    return await this.getCoercedOrDefault(createPendingTicketsCountKey(counterparty), Balance.deserialize, Balance.ZERO)
  }

  public async getLosingTicketCount(): Promise<number> {
    return await this.getCoercedOrDefault(LOSING_TICKET_COUNT, u8aToNumber, 0)
  }

  public async markPending(ticket: Ticket) {
    return await this.addBalance(createPendingTicketsCountKey(ticket.counterparty), ticket.amount)
  }

  public async resolvePending(ticket: Partial<Ticket>, snapshot: Snapshot) {
    let val = await this.getCoercedOrDefault<Balance>(
      createPendingTicketsCountKey(ticket.counterparty),
      Balance.deserialize,
      Balance.ZERO
    )

    await this.db
      .batch()
      .put(
        Buffer.from(this.keyOf(createPendingTicketsCountKey(ticket.counterparty))),
        Buffer.from(val.sub(val).serialize())
      )
      .put(Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY), Buffer.from(snapshot.serialize()))
      .write()
  }

  public async markRedeemeed(a: AcknowledgedTicket): Promise<void> {
    await this.increment(REDEEMED_TICKETS_COUNT)
    await this.delAcknowledgedTicket(a)
    await this.addBalance(REDEEMED_TICKETS_VALUE, a.ticket.amount)
    await this.subBalance(createPendingTicketsCountKey(a.ticket.counterparty), a.ticket.amount)
  }

  public async markLosing(t: UnacknowledgedTicket): Promise<void> {
    await this.increment(LOSING_TICKET_COUNT)
    await this.del(createPendingAcknowledgement(t.getChallenge()))
    await this.subBalance(createPendingTicketsCountKey(t.ticket.counterparty), t.ticket.amount)
  }

  public async getRejectedTicketsValue(): Promise<Balance> {
    return await this.getCoercedOrDefault(REJECTED_TICKETS_VALUE, Balance.deserialize, Balance.ZERO)
  }
  public async getRejectedTicketsCount(): Promise<number> {
    return this.getCoercedOrDefault(REJECTED_TICKETS_COUNT, u8aToNumber, 0)
  }
  public async markRejected(t: Ticket): Promise<void> {
    await this.increment(REJECTED_TICKETS_COUNT)
    await this.addBalance(REJECTED_TICKETS_VALUE, t.amount)
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

  public async setEnvironmentId(environment_id: string): Promise<void> {
    await this.put(ENVIRONMENT_KEY, encoder.encode(environment_id))
  }

  public async getEnvironmentId(): Promise<string> {
    return decoder.decode(await this.maybeGet(ENVIRONMENT_KEY))
  }

  public async verifyEnvironmentId(expectedId: string): Promise<boolean> {
    const storedId = await this.getEnvironmentId()

    if (storedId == undefined) {
      return false
    }

    return storedId === expectedId
  }

  public async getHoprBalance(): Promise<Balance> {
    return this.getCoercedOrDefault(HOPR_BALANCE_KEY, Balance.deserialize, Balance.ZERO)
  }

  public async setHoprBalance(value: Balance): Promise<void> {
    return this.put(HOPR_BALANCE_KEY, value.serialize())
  }

  public async addHoprBalance(value: Balance, snapshot: Snapshot): Promise<void> {
    let val = await this.getCoercedOrDefault<Balance>(HOPR_BALANCE_KEY, Balance.deserialize, Balance.ZERO)

    await this.db
      .batch()
      .put(Buffer.from(this.keyOf(HOPR_BALANCE_KEY)), Buffer.from(val.add(value).serialize()))
      .put(Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY), Buffer.from(snapshot.serialize()))
      .write()
  }

  public async subHoprBalance(value: Balance, snapshot: Snapshot): Promise<void> {
    let val = await this.getCoercedOrDefault<Balance>(HOPR_BALANCE_KEY, Balance.deserialize, Balance.ZERO)

    await this.db
      .batch()
      .put(Buffer.from(this.keyOf(HOPR_BALANCE_KEY)), Buffer.from(val.sub(value).serialize()))
      .put(Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY), Buffer.from(snapshot.serialize()))
      .write()
  }

  /**
   * Hopr Network Registry
   * Link hoprNode to an ETH address.
   * @param hoprNode the node to register
   * @param account the account that made the transaction
   * @param snapshot
   */
  public async addToRegistry(hoprNode: PublicKey, account: Address, snapshot: Snapshot): Promise<void> {
    await this.db
      .batch()
      .put(Buffer.from(this.keyOf(createWhitelistRegistryKey(hoprNode))), Buffer.from(account.serialize()))
      .put(Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY), Buffer.from(snapshot.serialize()))
      .write()
  }

  /**
   * Hopr Network Registry
   * Unlink hoprNode to an ETH address by removing the entry.
   * @param hoprNode the node to register
   * @param snapshot
   */
  public async removeFromRegistry(account: Address, snapshot: Snapshot): Promise<void> {
    // range of keys to search
    const from = this.keyOf(
      Uint8Array.from([...WHITELIST_REGISTRY_PREFIX, ...new Uint8Array(PublicKey.SIZE_UNCOMPRESSED).fill(0x00)])
    )
    const to = this.keyOf(
      Uint8Array.from([...WHITELIST_REGISTRY_PREFIX, ...new Uint8Array(PublicKey.SIZE_UNCOMPRESSED).fill(0xff)])
    )

    // create iterable stream to search all registered nodes
    const iterable = this.db.iterator({
      gte: Buffer.from(from),
      lte: Buffer.from(to),
      keys: true,
      values: true
    })

    let entryKey: Buffer | undefined
    // search for a matching account
    // we are interested in finding the `key`, this `.getAll` can't be used
    for await (const [key, val] of iterable as any) {
      try {
        if (account.eq(Address.deserialize(Buffer.from(val)))) {
          // get hoprNode from key
          entryKey = Buffer.from(key)
          break
        }
      } catch {}
    }

    if (entryKey) {
      await this.db
        .batch()
        .del(entryKey)
        .put(Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY), Buffer.from(snapshot.serialize()))
        .write()
    }
  }

  /**
   * Hopr Network Registry
   * Get address associated with hoprNode.
   * @param hoprNode the node to register
   * @returns ETH address
   */
  public async getAccountFromRegistry(hoprNode: PublicKey): Promise<Address> {
    return this.getCoerced<Address>(createWhitelistRegistryKey(hoprNode), Address.deserialize)
  }

  /**
   * Hopr Network Registry
   * Set address as eligible.
   * @param account the account that made the transaction
   * @param snapshot
   */
  public async setEligible(account: Address, eligible: boolean, snapshot: Snapshot): Promise<void> {
    const key = Buffer.from(this.keyOf(createWhitelistEligibleKey(account)))

    if (eligible) {
      await this.db
        .batch()
        .put(key, Buffer.from([]))
        .put(Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY), Buffer.from(snapshot.serialize()))
        .write()
    } else {
      await this.db
        .batch()
        .del(key)
        .put(Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY), Buffer.from(snapshot.serialize()))
        .write()
    }
  }

  /**
   * Hopr Network Registry
   * @param account the account that made the transaction
   * @returns true if account is eligible
   */
  public async isEligible(account: Address): Promise<boolean> {
    return this.getCoercedOrDefault(createWhitelistEligibleKey(account), () => true, false)
  }

  /**
   * Hopr Network Registry
   * @param enabled whether whitelist is enabled
   */
  public async setWhitelistEnabled(enabled: boolean, snapshot: Snapshot): Promise<void> {
    await this.db
      .batch()
      .put(Buffer.from(this.keyOf(WHITELIST_ENABLED_PREFIX)), Buffer.from([Number(enabled)]))
      .put(Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY), Buffer.from(snapshot.serialize()))
      .write()
  }

  /**
   * Hopr Network Registry
   * @returns true if whitelist is enabled
   */
  public async isWhitelistEnabled(): Promise<boolean> {
    return this.getCoercedOrDefault(WHITELIST_ENABLED_PREFIX, (v) => Boolean(v[0]), false)
  }

  static createMock(id?: PublicKey): HoprDB {
    const mock: HoprDB = {
      id: id ?? PublicKey.createMock(),
      db: levelup(MemDown())
    } as any
    Object.setPrototypeOf(mock, HoprDB.prototype)

    return mock
  }
}
