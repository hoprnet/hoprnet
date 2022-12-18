import levelup, { type LevelUp } from 'levelup'
import leveldown from 'leveldown'
import MemDown from 'memdown'
import { stat, mkdir, rm } from 'fs/promises'
import { debug } from '../process/index.js'
import { Intermediate } from '../crypto/index.js'
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
} from '../types/index.js'
import BN from 'bn.js'
import { u8aToNumber, u8aConcat, toU8a } from '../u8a/index.js'
import fs from 'fs'

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
const NETWORK_REGISTRY_HOPR_NODE_PREFIX: Uint8Array = encoder.encode('networkRegistry:hopr-node-')
const NETWORK_REGISTRY_ADDRESS_ELIGIBLE_PREFIX: Uint8Array = encoder.encode('networkRegistry:addressEligible-')
const NETWORK_REGISTRY_ADDRESS_PUBLIC_KEY_PREFIX: Uint8Array = encoder.encode('networkRegistry:addressPublicKey-')

const NETWORK_REGISTRY_ENABLED_PREFIX: Uint8Array = encoder.encode('networkRegistry:enabled')

// Max value 2**32 - 1
const DEFAULT_SERIALIZED_NUMBER_LENGTH = 4

function createChannelKey(channelId: Hash): Uint8Array {
  return Uint8Array.from([...CHANNEL_PREFIX, ...channelId.serialize()])
}
function createAccountKey(address: Address): Uint8Array {
  return Uint8Array.from([...ACCOUNT_PREFIX, ...address.serialize()])
}
function createCommitmentKey(channelId: Hash, iteration: number): Uint8Array {
  return Uint8Array.from([...COMMITMENT_PREFIX, ...channelId.serialize(), ...toU8a(iteration, 4)])
}
function createCurrentCommitmentKey(channelId: Hash): Uint8Array {
  return Uint8Array.from([...CURRENT_COMMITMENT_PREFIX, ...channelId.serialize()])
}
function createCurrentTicketIndexKey(channelId: Hash): Uint8Array {
  return Uint8Array.from([...TICKET_INDEX_PREFIX, ...channelId.serialize()])
}
function createPendingTicketsCountKey(address: Address): Uint8Array {
  return Uint8Array.from([...PENDING_TICKETS_COUNT, ...address.serialize()])
}
function createAcknowledgedTicketKey(challenge: EthereumChallenge, channelEpoch: UINT256): Uint8Array {
  return Uint8Array.from([...ACKNOWLEDGED_TICKETS_PREFIX, ...channelEpoch.serialize(), ...challenge.serialize()])
}
function createPendingAcknowledgement(halfKey: HalfKeyChallenge): Uint8Array {
  return Uint8Array.from([...PENDING_ACKNOWLEDGEMENTS_PREFIX, ...halfKey.serialize()])
}
function createPacketTagKey(tag: Uint8Array): Uint8Array {
  return Uint8Array.from([...PACKET_TAG_PREFIX, ...tag])
}

// Use compressed EC-points within entry key because canonical (uncompressed) representation
// is not needed and thus prevents decompression operations when converting from PeerId.
// This happens e.g. on newly established connections.
function createNetworkRegistryEntryKey(publicKey: PublicKey): Uint8Array {
  return Uint8Array.from([...NETWORK_REGISTRY_HOPR_NODE_PREFIX, ...publicKey.serializeCompressed()])
}
function createNetworkRegistryAddressEligibleKey(address: Address): Uint8Array {
  return Uint8Array.from([...NETWORK_REGISTRY_ADDRESS_ELIGIBLE_PREFIX, ...address.serialize()])
}
function createNetworkRegistryAddressToPublicKeyKey(address: Address) {
  return Uint8Array.from([...NETWORK_REGISTRY_ADDRESS_PUBLIC_KEY_PREFIX, ...address.serialize()])
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
    } else {
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
          throw new Error(`Database does not exist: ${dbPath}`)
        }
      }
    }

    // CommonJS / ESM issue
    // @ts-ignore
    this.db = levelup(leveldown(dbPath))

    // Fully initialize database
    await this.db.open()

    log(`namespacing db by public key: ${this.id.toCompressedPubKeyHex()}`)
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
    return u8aConcat(this.id.serializeCompressed(), ...segments)
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
    const keyU8a = this.keyOf(key)
    await this.db.put(
      Buffer.from(keyU8a.buffer, keyU8a.byteOffset, keyU8a.byteLength),
      Buffer.from(value.buffer, value.byteOffset, value.byteLength)
    )
  }

  public dumpDatabase(destFile: string) {
    log(`Dumping current database to ${destFile}`)
    let dumpFile = fs.createWriteStream(destFile, { flags: 'a' })
    this.db
      .createReadStream({ keys: true, keyAsBuffer: true, values: true, valueAsBuffer: true })
      .on('data', (d) => {
        // Skip the public key prefix in each key
        let key = (d.key as Buffer).subarray(PublicKey.SIZE_COMPRESSED)
        let keyString = ''
        let isHex = false
        let sawDelimiter = false
        for (const b of key) {
          if (!sawDelimiter && b >= 32 && b <= 126) {
            // Print sequences of ascii chars normally
            let cc = String.fromCharCode(b)
            keyString += (isHex ? ' ' : '') + cc
            isHex = false
            // Once a delimiter is encountered, always print as hex since then
            sawDelimiter = sawDelimiter || cc == '-' || cc == ':'
          } else {
            // Print sequences of non-ascii chars as hex
            keyString += (!isHex ? '0x' : '') + (b as number).toString(16)
            isHex = true
          }
        }
        dumpFile.write(keyString + ':' + d.value.toString('hex') + '\n')
      })
      .on('end', function () {
        dumpFile.close()
      })
  }

  private async touch(key: Uint8Array): Promise<void> {
    return await this.put(key, new Uint8Array())
  }

  protected async get(key: Uint8Array): Promise<Uint8Array> {
    const keyU8a = this.keyOf(key)
    const value = await this.db.get(Buffer.from(keyU8a.buffer, keyU8a.byteOffset, keyU8a.byteLength))

    return new Uint8Array(value.buffer, value.byteOffset, value.byteLength)
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
      gte: Buffer.from(firstPrefixed.buffer, firstPrefixed.byteOffset, firstPrefixed.byteLength),
      lte: Buffer.from(lastPrefixed.buffer, lastPrefixed.byteOffset, lastPrefixed.byteLength),
      keys: false
    }) as any) {
      const obj: Element = deserialize(new Uint8Array(chunk.buffer, chunk.byteOffset, chunk.byteLength))

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

  protected async *getAllIterable<Element, TransformedElement = Element>(
    range: {
      prefix: Uint8Array
      suffixLength: number
    },
    deserialize: (u: Uint8Array) => Element,
    filter?: ((o: Element) => boolean) | undefined,
    map?: (i: Element) => TransformedElement
  ): AsyncIterable<TransformedElement> {
    const firstPrefixed = this.keyOf(range.prefix, new Uint8Array(range.suffixLength).fill(0x00))
    const lastPrefixed = this.keyOf(range.prefix, new Uint8Array(range.suffixLength).fill(0xff))

    // @TODO fix types in @types/levelup package
    for await (const [_key, chunk] of this.db.iterator({
      gte: Buffer.from(firstPrefixed.buffer, firstPrefixed.byteOffset, firstPrefixed.byteLength),
      lte: Buffer.from(lastPrefixed.buffer, lastPrefixed.byteOffset, lastPrefixed.byteLength),
      keys: false
    }) as any) {
      const obj: Element = deserialize(new Uint8Array(chunk.buffer, chunk.byteOffset, chunk.byteLength))

      if (!filter || filter(obj)) {
        if (map) {
          yield map(obj)
        } else {
          yield obj as unknown as TransformedElement
        }
      }
    }
  }

  private async del(key: Uint8Array): Promise<void> {
    const keyU8a = this.keyOf(key)
    await this.db.del(Buffer.from(keyU8a.buffer, keyU8a.byteOffset, keyU8a.byteLength))
  }

  private async increment(key: Uint8Array): Promise<number> {
    let val = await this.getCoercedOrDefault<number>(key, u8aToNumber, 0)
    // Always store using 4 bytes, max value 2**32 - 1
    await this.put(key, toU8a(val + 1, DEFAULT_SERIALIZED_NUMBER_LENGTH))
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
    return await this.getCoerced<PendingAckowledgement>(
      createPendingAcknowledgement(halfKeyChallenge),
      deserializePendingAcknowledgement
    )
  }

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

    const tickets: AcknowledgedTicket[] = []

    for await (const ticket of this.getAllIterable<AcknowledgedTicket>(
      {
        prefix: ACKNOWLEDGED_TICKETS_PREFIX,
        suffixLength: EthereumChallenge.SIZE
      },
      AcknowledgedTicket.deserialize,
      filterFunc
    )) {
      tickets.push(ticket)
    }

    return tickets.sort(sortFunc)
  }

  /**
   * Deletes all acknowledged tickets in a channel and updates
   * neglected tickets counter.
   * @param channel in which channel to delete tickets
   */
  public async deleteAcknowledgedTicketsFromChannel(channel: ChannelEntry): Promise<void> {
    const tickets = await this.getAcknowledgedTickets({ signer: channel.source })

    const neglectedTicketsCount = await this.getCoercedOrDefault<number>(NEGLECTED_TICKET_COUNT, u8aToNumber, 0)

    const batch = this.db.batch()

    for (const ack of tickets) {
      const keyU8a = this.keyOf(createAcknowledgedTicketKey(ack.ticket.challenge, ack.ticket.channelEpoch))
      batch.del(Buffer.from(keyU8a.buffer, keyU8a.byteOffset, keyU8a.byteLength))
    }

    // only update count if there has been a change
    if (tickets.length > 0) {
      batch.put(
        Buffer.from(this.keyOf(NEGLECTED_TICKET_COUNT)),
        // store updated number in 4 bytes
        Buffer.from(toU8a(neglectedTicketsCount + tickets.length, 4))
      )
    }

    await batch.write()
  }

  /**
   * Deletes an acknowledged ticket in database
   * @param ack acknowledged ticket
   */
  public async delAcknowledgedTicket(ack: AcknowledgedTicket): Promise<void> {
    await this.del(createAcknowledgedTicketKey(ack.ticket.challenge, ack.ticket.channelEpoch))
  }

  public async replaceUnAckWithAck(halfKeyChallenge: HalfKeyChallenge, ackTicket: AcknowledgedTicket): Promise<void> {
    const unAcknowledgedDbKey = this.keyOf(createPendingAcknowledgement(halfKeyChallenge))
    const acknowledgedDbKey = this.keyOf(
      createAcknowledgedTicketKey(ackTicket.ticket.challenge, ackTicket.ticket.channelEpoch)
    )

    const serializedTicket = ackTicket.serialize()

    await this.db
      .batch()
      .del(Buffer.from(unAcknowledgedDbKey.buffer, unAcknowledgedDbKey.byteOffset, unAcknowledgedDbKey.byteLength))
      .put(
        Buffer.from(acknowledgedDbKey.buffer, acknowledgedDbKey.byteOffset, acknowledgedDbKey.byteLength),
        Buffer.from(serializedTicket.buffer, serializedTicket.byteOffset, serializedTicket.byteLength)
      )
      .write()
  }

  /**
   * Get tickets, both unacknowledged and acknowledged
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
      const u8aKey = this.keyOf(createCommitmentKey(channelId, intermediate.iteration))

      dbBatch = dbBatch.put(
        Buffer.from(u8aKey.buffer, u8aKey.byteOffset, u8aKey.byteLength),
        Buffer.from(intermediate.preImage.buffer, intermediate.preImage.byteOffset, intermediate.preImage.byteLength)
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
    return await this.getCoercedOrDefault<UINT256>(
      createCurrentTicketIndexKey(channelId),
      UINT256.deserialize,
      undefined
    )
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
    return await this.getCoercedOrDefault<Snapshot>(LATEST_CONFIRMED_SNAPSHOT_KEY, Snapshot.deserialize, undefined)
  }

  async updateLatestConfirmedSnapshot(snapshot: Snapshot): Promise<void> {
    await this.put(LATEST_CONFIRMED_SNAPSHOT_KEY, snapshot.serialize())
  }

  async getChannel(channelId: Hash): Promise<ChannelEntry> {
    return await this.getCoerced<ChannelEntry>(createChannelKey(channelId), ChannelEntry.deserialize)
  }

  async *getChannelsIterable(filter?: (channel: ChannelEntry) => boolean): AsyncIterable<ChannelEntry> {
    yield* this.getAllIterable<ChannelEntry>(
      {
        prefix: CHANNEL_PREFIX,
        suffixLength: Hash.SIZE
      },
      ChannelEntry.deserialize,
      filter
    )
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
    const serializedChannel = channel.serialize()
    const keyU8a = this.keyOf(createChannelKey(channelId))

    const serializedSnapshot = snapshot.serialize()

    await this.db
      .batch()
      .put(
        Buffer.from(keyU8a.buffer, keyU8a.byteOffset, keyU8a.byteLength),
        Buffer.from(serializedChannel.buffer, serializedChannel.byteOffset, serializedChannel.byteLength)
      )
      .put(
        Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY),
        Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
      )
      .write()
  }

  async getAccount(address: Address): Promise<AccountEntry | undefined> {
    return await this.getCoercedOrDefault<AccountEntry>(createAccountKey(address), AccountEntry.deserialize, undefined)
  }

  async updateAccountAndSnapshot(account: AccountEntry, snapshot: Snapshot): Promise<void> {
    const serializedAccount = account.serialize()
    const serializedSnapshot = snapshot.serialize()

    await this.db
      .batch()
      .put(
        Buffer.from(this.keyOf(createAccountKey(account.getAddress()))),
        Buffer.from(serializedAccount.buffer, serializedAccount.byteOffset, serializedAccount.byteLength)
      )
      .put(
        Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY),
        Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
      )
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

  async *getAccountsIterable(filter?: (account: AccountEntry) => boolean) {
    yield* this.getAllIterable<AccountEntry>(
      {
        prefix: ACCOUNT_PREFIX,
        suffixLength: Address.SIZE
      },
      AccountEntry.deserialize,
      filter
    )
  }

  public async getRedeemedTicketsValue(): Promise<Balance> {
    return await this.getCoercedOrDefault<Balance>(REDEEMED_TICKETS_VALUE, Balance.deserialize, Balance.ZERO)
  }
  public async getRedeemedTicketsCount(): Promise<number> {
    return this.getCoercedOrDefault<number>(REDEEMED_TICKETS_COUNT, u8aToNumber, 0)
  }

  public async getNeglectedTicketsCount(): Promise<number> {
    return this.getCoercedOrDefault<number>(NEGLECTED_TICKET_COUNT, u8aToNumber, 0)
  }

  public async getPendingTicketCount(): Promise<number> {
    return (await this.getUnacknowledgedTickets()).length
  }

  public async getPendingBalanceTo(counterparty: Address): Promise<Balance> {
    return await this.getCoercedOrDefault<Balance>(
      createPendingTicketsCountKey(counterparty),
      Balance.deserialize,
      Balance.ZERO
    )
  }

  public async getLosingTicketCount(): Promise<number> {
    return await this.getCoercedOrDefault<number>(LOSING_TICKET_COUNT, u8aToNumber, 0)
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

    const serializedSnapshot = snapshot.serialize()
    const u8aPendingKey = this.keyOf(createPendingTicketsCountKey(ticket.counterparty))

    await this.db
      .batch()
      .put(
        Buffer.from(u8aPendingKey.buffer, u8aPendingKey.byteOffset, u8aPendingKey.byteLength),
        Buffer.from(val.sub(val).serialize())
      )
      .put(
        Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY),
        Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
      )
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
    return await this.getCoercedOrDefault<Balance>(REJECTED_TICKETS_VALUE, Balance.deserialize, Balance.ZERO)
  }
  public async getRejectedTicketsCount(): Promise<number> {
    return this.getCoercedOrDefault<number>(REJECTED_TICKETS_COUNT, u8aToNumber, 0)
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

  public async *getChannelsFromIterable(address: Address) {
    for await (const channel of this.getChannelsIterable()) {
      if (address.eq(channel.source.toAddress())) {
        yield channel
      }
    }
  }

  public async getChannelsTo(address: Address) {
    return this.getChannels((channel) => {
      return address.eq(channel.destination.toAddress())
    })
  }

  public async *getChannelsToIterable(address: Address) {
    for await (const channel of this.getChannelsIterable()) {
      if (address.eq(channel.destination.toAddress())) {
        yield channel
      }
    }
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
    return this.getCoercedOrDefault<Balance>(HOPR_BALANCE_KEY, Balance.deserialize, Balance.ZERO)
  }

  public async setHoprBalance(value: Balance): Promise<void> {
    return this.put(HOPR_BALANCE_KEY, value.serialize())
  }

  public async addHoprBalance(value: Balance, snapshot: Snapshot): Promise<void> {
    const val = await this.getCoercedOrDefault<Balance>(HOPR_BALANCE_KEY, Balance.deserialize, Balance.ZERO)

    const serializedSnapshot = snapshot.serialize()

    await this.db
      .batch()
      .put(Buffer.from(this.keyOf(HOPR_BALANCE_KEY)), Buffer.from(val.add(value).serialize()))
      .put(
        Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY),
        Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
      )
      .write()
  }

  public async subHoprBalance(value: Balance, snapshot: Snapshot): Promise<void> {
    const val = await this.getCoercedOrDefault<Balance>(HOPR_BALANCE_KEY, Balance.deserialize, Balance.ZERO)

    const serializedSnapshot = snapshot.serialize()

    await this.db
      .batch()
      .put(Buffer.from(this.keyOf(HOPR_BALANCE_KEY)), Buffer.from(val.sub(value).serialize()))
      .put(
        Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY),
        Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
      )
      .write()
  }

  /**
   * Hopr Network Registry
   * Link hoprNode to an ETH address.
   * @param pubKey the node to register
   * @param account the account that made the transaction
   * @param snapshot
   */
  public async addToNetworkRegistry(pubKey: PublicKey, account: Address, snapshot: Snapshot): Promise<void> {
    let registeredNodes = []
    try {
      registeredNodes = await this.findHoprNodesUsingAccountInNetworkRegistry(account)
    } catch (error) {}

    // add new node to the list
    registeredNodes.push(pubKey)

    const serializedRegisteredNodes = PublicKey.serializeArray(registeredNodes)
    const serializedSnapshot = snapshot.serialize()
    const serializedAccount = account.serialize()

    await this.db
      .batch()
      // node public key to address (M->1)
      .put(
        Buffer.from(this.keyOf(createNetworkRegistryEntryKey(pubKey))),
        Buffer.from(serializedAccount.buffer, serializedAccount.byteOffset, serializedAccount.byteLength)
      )
      // address to node public keys (1->M) in the format of key -> PublicKey[]
      .put(
        Buffer.from(this.keyOf(createNetworkRegistryAddressToPublicKeyKey(account))),
        Buffer.from(
          serializedRegisteredNodes.buffer,
          serializedRegisteredNodes.byteOffset,
          serializedRegisteredNodes.byteLength
        )
      )
      .put(
        Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY),
        Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
      )
      .write()
  }

  /**
   * Do a reverse find by searching the stored account to return
   * the associated public keys of registered HOPR nodes.
   * @param account
   * @returns array of PublicKey of the associated HOPR nodes
   */
  public async findHoprNodesUsingAccountInNetworkRegistry(account: Address): Promise<PublicKey[]> {
    const pubKeys = await this.getCoercedOrDefault<PublicKey[]>(
      createNetworkRegistryAddressToPublicKeyKey(account),
      PublicKey.deserializeArray,
      undefined
    )

    if (!pubKeys) {
      throw Error('HoprNode not found')
    }

    return pubKeys
  }

  /**
   * Hopr Network Registry
   * Unlink hoprNode to an ETH address by removing the entry.
   * @param pubKey the node's x
   * @param account the account to use so we can search for the key in the database
   * @param snapshot
   */
  public async removeFromNetworkRegistry(pubKey: PublicKey, account: Address, snapshot: Snapshot): Promise<void> {
    let registeredNodes = []
    try {
      registeredNodes = await this.findHoprNodesUsingAccountInNetworkRegistry(account)
    } catch (error) {
      log(`cannot remove node from network registry due to ${error}`)
      throw Error('HoprNode not registered to the account')
    }

    // find registered peer id index
    const registeredIndex = registeredNodes.findIndex((registeredPubKey) => pubKey.eq(registeredPubKey))

    if (registeredIndex < 0) {
      log(`cannot remove node from network registry, not found`)
      throw Error('HoprNode not registered to the account')
    }

    // remove nodes
    registeredNodes.splice(registeredIndex, 1)

    const entryKey = createNetworkRegistryEntryKey(pubKey)

    if (entryKey) {
      const serializedRegisteredNodes = PublicKey.serializeArray(registeredNodes)
      const serializedSnapshot = snapshot.serialize()

      await this.db
        .batch()
        .del(Buffer.from(this.keyOf(entryKey)))
        // address to node public keys (1->M) in the format of key -> PublicKey[]
        .put(
          Buffer.from(this.keyOf(createNetworkRegistryAddressToPublicKeyKey(account))),
          Buffer.from(
            serializedRegisteredNodes.buffer,
            serializedRegisteredNodes.byteOffset,
            serializedRegisteredNodes.byteLength
          )
        )
        .put(
          Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY),
          Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
        )
        .write()
    }
  }

  /**
   * Hopr Network Registry
   * Get address associated with hoprNode.
   * @param hoprNode the node to register
   * @returns ETH address
   */
  public async getAccountFromNetworkRegistry(hoprNode: PublicKey): Promise<Address> {
    return this.getCoerced<Address>(createNetworkRegistryEntryKey(hoprNode), Address.deserialize)
  }

  /**
   * Hopr Network Registry
   * Set address as eligible.
   * @param account the account that made the transaction
   * @param snapshot
   */
  public async setEligible(account: Address, eligible: boolean, snapshot: Snapshot): Promise<void> {
    const key = Buffer.from(this.keyOf(createNetworkRegistryAddressEligibleKey(account)))

    const serializedSnapshot = snapshot.serialize()

    if (eligible) {
      await this.db
        .batch()
        .put(key, Buffer.from([]))
        .put(
          Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY),
          Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
        )
        .write()
    } else {
      await this.db
        .batch()
        .del(key)
        .put(
          Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY),
          Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
        )
        .write()
    }
  }

  /**
   * Hopr Network Registry
   * @param account the account that made the transaction
   * @returns true if account is eligible
   */
  public async isEligible(account: Address): Promise<boolean> {
    return this.getCoercedOrDefault<boolean>(createNetworkRegistryAddressEligibleKey(account), () => true, false)
  }

  /**
   * Hopr Network Registry
   * @param enabled whether register is enabled
   */
  public async setNetworkRegistryEnabled(enabled: boolean, snapshot: Snapshot): Promise<void> {
    const serializedSnapshot = snapshot.serialize()

    await this.db
      .batch()
      .put(Buffer.from(this.keyOf(NETWORK_REGISTRY_ENABLED_PREFIX)), Buffer.from([enabled ? 1 : 0]))
      .put(
        Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY),
        Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
      )
      .write()
  }

  /**
   * Check ifs Network registry is enabled
   * @returns true if register is enabled or if key is not preset in the dababase
   */
  public async isNetworkRegistryEnabled(): Promise<boolean> {
    return this.getCoercedOrDefault<boolean>(NETWORK_REGISTRY_ENABLED_PREFIX, (v) => v[0] != 0, true)
  }

  static createMock(id?: PublicKey): HoprDB {
    const mock: HoprDB = {
      id: id ?? PublicKey.createMock(),
      // CommonJS / ESM issue
      // @ts-ignore
      db: levelup(MemDown())
    } as any
    Object.setPrototypeOf(mock, HoprDB.prototype)

    return mock
  }
}
