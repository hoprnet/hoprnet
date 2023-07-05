import levelup, { type LevelUp } from 'levelup'
import leveldown from 'leveldown'
import MemDown from 'memdown'
import { stat, mkdir, rm } from 'fs/promises'
import { debug } from '../process/index.js'

// import {
//   AcknowledgedTicket,
//   UnacknowledgedTicket,
//   AccountEntry,
//   ChannelEntry,
//   Snapshot,
//   PublicKey,
//   Balance,
//   HalfKeyChallenge,
//   EthereumChallenge,
//   U256,
//   Ticket,
//   Address,
//   Hash,
//   generate_channel_id,
//   BalanceType,
//   PendingAcknowledgement
// } from '../types.js'
// import BN from 'bn.js'
import fs from 'fs'
import { u8aConcat, u8aToHex } from '../u8a/index.js'
// import type { IteratedHash } from '../../../core/lib/core_crypto.js'

const log = debug(`hopr-core:db`)

const encoder = new TextEncoder()
const decoder = new TextDecoder()

// key seperator:    `:`
// key query prefix: `-`

// const ACCOUNT_PREFIX = encoder.encode('account-')
// const CHANNEL_PREFIX = encoder.encode('channel-')
// const COMMITMENT_PREFIX = encoder.encode('commitment-')
// const CURRENT_COMMITMENT_PREFIX = encoder.encode('commitment:current-')
// const TICKET_INDEX_PREFIX = encoder.encode('ticketIndex-')
// const PENDING_TICKETS_COUNT = encoder.encode('statistics:pending:value-')
// const ACKNOWLEDGED_TICKETS_PREFIX = encoder.encode('tickets:acknowledged-')
// const PENDING_ACKNOWLEDGEMENTS_PREFIX = encoder.encode('tickets:pending-acknowledgement-')
// const PACKET_TAG_PREFIX: Uint8Array = encoder.encode('packets:tag-')
// const NETWORK_REGISTRY_HOPR_NODE_PREFIX: Uint8Array = encoder.encode('networkRegistry:hopr-node-')
// const NETWORK_REGISTRY_ADDRESS_ELIGIBLE_PREFIX: Uint8Array = encoder.encode('networkRegistry:addressEligible-')
// const NETWORK_REGISTRY_ADDRESS_PUBLIC_KEY_PREFIX: Uint8Array = encoder.encode('networkRegistry:addressPublicKey-')

// const NETWORK_REGISTRY_ENABLED_PREFIX: Uint8Array = encoder.encode('networkRegistry:enabled')

// const GENERIC_OBJECT_PREFIX: Uint8Array = encoder.encode('genericObjects:')

// Max value 2**32 - 1
// const DEFAULT_SERIALIZED_NUMBER_LENGTH = 4

// function createChannelKey(channelId: Hash): Uint8Array {
//   return Uint8Array.from([...CHANNEL_PREFIX, ...channelId.serialize()])
// }
// function createAccountKey(address: Address): Uint8Array {
//   return Uint8Array.from([...ACCOUNT_PREFIX, ...address.serialize()])
// }
// function createCommitmentKey(channelId: Hash, iteration: number): Uint8Array {
//   return Uint8Array.from([...COMMITMENT_PREFIX, ...channelId.serialize(), ...toU8a(iteration, 4)])
// }
// function createCurrentCommitmentKey(channelId: Hash): Uint8Array {
//   return Uint8Array.from([...CURRENT_COMMITMENT_PREFIX, ...channelId.serialize()])
// }
// function createCurrentTicketIndexKey(channelId: Hash): Uint8Array {
//   return Uint8Array.from([...TICKET_INDEX_PREFIX, ...channelId.serialize()])
// }
// function createPendingTicketsCountKey(address: Address): Uint8Array {
//   return Uint8Array.from([...PENDING_TICKETS_COUNT, ...address.serialize()])
// }
// function createAcknowledgedTicketKey(challenge: EthereumChallenge, channelEpoch: U256): Uint8Array {
//   return Uint8Array.from([...ACKNOWLEDGED_TICKETS_PREFIX, ...channelEpoch.serialize(), ...challenge.serialize()])
// }
// function createPendingAcknowledgement(halfKey: HalfKeyChallenge): Uint8Array {
//   return Uint8Array.from([...PENDING_ACKNOWLEDGEMENTS_PREFIX, ...halfKey.serialize()])
// }
// function createPacketTagKey(tag: Uint8Array): Uint8Array {
//   return Uint8Array.from([...PACKET_TAG_PREFIX, ...tag])
// }
// function createObjectKey(namespace: string, key: string) {
//   const encodedNs = encoder.encode(`${namespace}:`)
//   const encodedKey = encoder.encode(key)
//   return Uint8Array.from([...GENERIC_OBJECT_PREFIX, ...encodedNs, ...encodedKey])
// }

// Use compressed EC-points within entry key because canonical (uncompressed) representation
// is not needed and thus prevents decompression operations when converting from PeerId.
// This happens e.g. on newly established connections.
// function createNetworkRegistryEntryKey(publicKey: PublicKey): Uint8Array {
//   return Uint8Array.from([...NETWORK_REGISTRY_HOPR_NODE_PREFIX, ...publicKey.serialize(true)])
// }
// function createNetworkRegistryAddressEligibleKey(address: Address): Uint8Array {
//   return Uint8Array.from([...NETWORK_REGISTRY_ADDRESS_ELIGIBLE_PREFIX, ...address.serialize()])
// }
// function createNetworkRegistryAddressToPublicKeyKey(address: Address) {
//   return Uint8Array.from([...NETWORK_REGISTRY_ADDRESS_PUBLIC_KEY_PREFIX, ...address.serialize()])
// }

// const LATEST_BLOCK_NUMBER_KEY = encoder.encode('latestBlockNumber')
// const LATEST_CONFIRMED_SNAPSHOT_KEY = encoder.encode('latestConfirmedSnapshot')
// const REDEEMED_TICKETS_COUNT = encoder.encode('statistics:redeemed:count')
// const REDEEMED_TICKETS_VALUE = encoder.encode('statistics:redeemed:value')
// const LOSING_TICKET_COUNT = encoder.encode('statistics:losing:count')
// const NEGLECTED_TICKET_COUNT = encoder.encode('statistics:neglected:count')
// const REJECTED_TICKETS_COUNT = encoder.encode('statistics:rejected:count')
// const REJECTED_TICKETS_VALUE = encoder.encode('statistics:rejected:value')
const NETWORK_KEY = encoder.encode('network_id')
// const HOPR_BALANCE_KEY = encoder.encode('hopr-balance')

// enum Secp256k1PublicKeyPrefix {
//   COMPRESSED_NEGATIVE = 0x02,
//   COMPRESSED_POSITIVE = 0x03,
//   UNCOMPRESSED = 0x04
// }

export class LevelDb {
  public backend: LevelUp

  constructor() {
    // unless initialized with a specific db path, memory version is used
    this.backend = new levelup(MemDown())
  }

  public async init(initialize: boolean, dbPath: string, forceCreate: boolean = false, networkId: string) {
    let setNetwork = false

    log(`using db at ${dbPath}`)
    if (forceCreate) {
      log('force create - wipe old database and create a new')
      await rm(dbPath, { recursive: true, force: true })
      await mkdir(dbPath, { recursive: true })
      setNetwork = true
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
          setNetwork = true
        } else {
          throw new Error(`Database does not exist: ${dbPath}`)
        }
      }
    }

    // CommonJS / ESM issue
    // @ts-ignore
    this.backend = levelup(leveldown(dbPath))

    // Fully initialize database
    await this.backend.open()

    if (setNetwork) {
      log(`setting network id ${networkId} to db`)
      await this.put(NETWORK_KEY, encoder.encode(networkId))
    } else {
      let storedNetworkId = await this.maybeGet(NETWORK_KEY)
      let decodedStoredNetworkId = storedNetworkId !== undefined ? undefined : decoder.decode(storedNetworkId)

      const hasNetworkKey = decodedStoredNetworkId !== undefined && decodedStoredNetworkId === networkId

      if (!hasNetworkKey) {
        throw new Error(`invalid db network id: ${decodedStoredNetworkId} (expected: ${networkId})`)
      }
    }
  }

  public async put(key: Uint8Array, value: Uint8Array): Promise<void> {
    // LevelDB does not support Uint8Arrays, always convert to Buffer
    return await this.backend.put(
      Buffer.from(key.buffer, key.byteOffset, key.byteLength),
      Buffer.from(value.buffer, value.byteOffset, value.byteLength)
    )
  }

  public async get(key: Uint8Array): Promise<Uint8Array> {
    // LevelDB does not support Uint8Arrays, always convert to Buffer
    const value = await this.backend.get(Buffer.from(key.buffer, key.byteOffset, key.byteLength))

    return new Uint8Array(value.buffer, value.byteOffset, value.byteLength)
  }

  public async remove(key: Uint8Array): Promise<void> {
    await this.backend.del(Buffer.from(key.buffer, key.byteOffset, key.byteLength))
  }

  public async batch(ops: Array<any>, wait_for_write = true): Promise<void> {
    const options: { sync: boolean } = {
      sync: wait_for_write
    }

    let batch = this.backend.batch()
    for (const op of ops) {
      if (!op.hasOwnProperty('type') || !op.hasOwnProperty('key')) {
        throw new Error('Invalid operation, missing key or type: ' + JSON.stringify(op))
      }

      if (op.type === 'put') {
        // LevelDB does not support Uint8Arrays, always convert to Buffer
        batch.put(
          Buffer.from(op.key, op.key.byteOffset, op.key.byteLength),
          Buffer.from(op.value, op.value.byteOffset, op.value.byteLength)
        )
      } else if (op.type === 'del') {
        // LevelDB does not support Uint8Arrays, always convert to Buffer
        batch.del(Buffer.from(op.key, op.key.byteOffset, op.key.byteLength))
      } else {
        throw new Error(`Unsupported operation type: ${JSON.stringify(op)}`)
      }
    }

    await batch.write(options)
  }

  public async maybeGet(key: Uint8Array): Promise<Uint8Array | undefined> {
    try {
      // Conversion to Buffer done by `.get()` method
      return await this.get(key)
    } catch (err) {
      if (err.type === 'NotFoundError' || err.notFound) {
        return undefined
      }
      throw err
    }
  }

  public iterValues(prefix: Uint8Array, suffixLength: number): AsyncIterable<Uint8Array> {
    return this.iter(prefix, suffixLength)
  }

  protected async *iter(prefix: Uint8Array, suffixLength: number): AsyncIterable<Uint8Array> {
    const firstPrefixed = u8aConcat(prefix, new Uint8Array(suffixLength).fill(0x00))
    const lastPrefixed = u8aConcat(prefix, new Uint8Array(suffixLength).fill(0xff))

    for await (const [_key, chunk] of this.backend.iterator({
      // LevelDB does not support Uint8Arrays, always convert to Buffer
      gte: Buffer.from(firstPrefixed.buffer, firstPrefixed.byteOffset, firstPrefixed.byteLength),
      lte: Buffer.from(lastPrefixed.buffer, lastPrefixed.byteOffset, lastPrefixed.byteLength),
      keys: false
    }) as any) {
      const obj: Uint8Array = new Uint8Array(chunk.buffer, chunk.byteOffset, chunk.byteLength)

      yield obj
    }
  }

  public async close() {
    log('Closing database')
    return await this.backend.close()
  }

  public dump(destFile: string) {
    log(`Dumping current database to ${destFile}`)
    let dumpFile = fs.createWriteStream(destFile, { flags: 'a' })
    this.backend
      .createReadStream({ keys: true, keyAsBuffer: true, values: false, valueAsBuffer: true })
      .on('data', ({ key }) => {
        let out = ''
        while (key.length > 0) {
          const nextDelimiter = key.findIndex((v: number) => v == 0x2d) // 0x2d ~= '-'

          if (key.subarray(0, nextDelimiter).every((v: number) => v >= 32 && v <= 126)) {
            out += decoder.decode(key.subarray(0, nextDelimiter))
          } else {
            out += u8aToHex(key.subarray(0, nextDelimiter))
          }

          if (nextDelimiter < 0) {
            break
          } else {
            key = (key as Buffer).subarray(nextDelimiter + 1)
          }
        }

        dumpFile.write(out + '\n')
      })
      .on('end', function () {
        dumpFile.close()
      })
  }

  public async setNetworkId(network_id: string): Promise<void> {
    // conversion to Buffer done by `.put()` method
    await this.put(NETWORK_KEY, encoder.encode(network_id))
  }

  public async getNetworkId(): Promise<string> {
    // conversion to Buffer done by `.get()` method
    return decoder.decode(await this.maybeGet(NETWORK_KEY))
  }

  public async verifyNetworkId(expectedId: string): Promise<boolean> {
    const storedId = await this.getNetworkId()

    if (storedId == undefined) {
      return false
    }

    return storedId === expectedId
  }
}

/// Class designated to migrate the functionality to Rust
// export class HoprDB {
//   // made public to allow access for Rust code
//   public db: LevelDb
//   public id: PublicKey

//   constructor(publicKey: PublicKey) {
//     this.db = new LevelDb()
//     this.id = publicKey
//   }

//   // ---------- direct access API to be removed after migration to Rust

//   async init(initialize: boolean, dbPath: string, forceCreate: boolean = false, networkId: string) {
//     await this.db.init(initialize, dbPath, forceCreate, networkId)
//   }

//   protected async put(key: Uint8Array, value: Uint8Array): Promise<void> {
//     // LevelDB does not support Uint8Arrays, always convert to Buffer
//     return await this.db.put(
//       Buffer.from(key.buffer, key.byteOffset, key.byteLength),
//       Buffer.from(value.buffer, value.byteOffset, value.byteLength)
//     )
//   }

//   protected async get(key: Uint8Array): Promise<Uint8Array> {
//     // LevelDB does not support Uint8Arrays, always convert to Buffer
//     const value = await this.db.get(Buffer.from(key.buffer, key.byteOffset, key.byteLength))

//     // LevelDB always outputs Buffer, so convert to Uint8Array
//     return new Uint8Array(value.buffer, value.byteOffset, value.byteLength)
//   }

//   public async remove(key: Uint8Array): Promise<void> {
//     await this.db.remove(key)
//   }

//   public dumpDatabase(destFile: string) {
//     this.db.dump(destFile)
//   }

//   public async close() {
//     await this.db.close()
//   }

//   // ---------- end direct access API to be removed after migration to Rust

//   private async getCoerced<T>(key: Uint8Array, coerce: (u: Uint8Array) => T) {
//     let u8a = await this.db.get(key)
//     return coerce(u8a)
//   }

//   private async getCoercedOrDefault<T>(key: Uint8Array, coerce: (u: Uint8Array) => T, defaultVal: T) {
//     let u8a = await this.db.maybeGet(key)
//     if (u8a === undefined) {
//       return defaultVal
//     }
//     return coerce(u8a)
//   }

//   /**
//    * Gets a elements from the database of a kind.
//    * Optionally applies `filter`then `map` then `sort` to the result.
//    * @param range.prefix key prefix, such as `channels-`
//    * @param range.suffixLength length of the appended identifier to distinguish elements
//    * @param deserialize function to parse serialized objects
//    * @param filter [optional] filter deserialized objects
//    * @param map [optional] transform deserialized and filtered objects
//    * @param sorter [optional] sort deserialized, filtered and transformed objects
//    * @returns a Promises that resolves with the found elements
//    */
//   protected async getAll<Element, TransformedElement = Element>(
//     range: {
//       prefix: Uint8Array
//       suffixLength: number
//     },
//     deserialize: (u: Uint8Array) => Element,
//     filter?: ((o: Element) => boolean) | undefined,
//     map?: (i: Element) => TransformedElement,
//     sorter?: (e1: TransformedElement, e2: TransformedElement) => number
//   ): Promise<TransformedElement[]> {
//     const firstPrefixed = u8aConcat(range.prefix, new Uint8Array(range.suffixLength).fill(0x00))
//     const lastPrefixed = u8aConcat(range.prefix, new Uint8Array(range.suffixLength).fill(0xff))

//     const results: TransformedElement[] = []

//     // @TODO fix types in @types/levelup package
//     for await (const [_key, chunk] of this.db.backend.iterator({
//       gte: Buffer.from(firstPrefixed.buffer, firstPrefixed.byteOffset, firstPrefixed.byteLength),
//       lte: Buffer.from(lastPrefixed.buffer, lastPrefixed.byteOffset, lastPrefixed.byteLength),
//       keys: false
//     }) as any) {
//       const obj: Element = deserialize(new Uint8Array(chunk.buffer, chunk.byteOffset, chunk.byteLength))

//       if (!filter || filter(obj)) {
//         if (map) {
//           results.push(map(obj))
//         } else {
//           results.push(obj as unknown as TransformedElement)
//         }
//       }
//     }

//     if (sorter) {
//       // sort in-place
//       results.sort(sorter)
//     }

//     return results
//   }

//   protected async *getAllIterable<Element, TransformedElement = Element>(
//     range: {
//       prefix: Uint8Array
//       suffixLength: number
//     },
//     deserialize: (u: Uint8Array) => Element,
//     filter?: ((o: Element) => boolean) | undefined,
//     map?: (i: Element) => TransformedElement
//   ): AsyncIterable<TransformedElement> {
//     const firstPrefixed = u8aConcat(range.prefix, new Uint8Array(range.suffixLength).fill(0x00))
//     const lastPrefixed = u8aConcat(range.prefix, new Uint8Array(range.suffixLength).fill(0xff))

//     // @TODO fix types in @types/levelup package
//     for await (const [_key, chunk] of this.db.backend.iterator({
//       gte: Buffer.from(firstPrefixed.buffer, firstPrefixed.byteOffset, firstPrefixed.byteLength),
//       lte: Buffer.from(lastPrefixed.buffer, lastPrefixed.byteOffset, lastPrefixed.byteLength),
//       keys: false
//     }) as any) {
//       const obj: Element = deserialize(new Uint8Array(chunk.buffer, chunk.byteOffset, chunk.byteLength))

//       if (!filter || filter(obj)) {
//         if (map) {
//           yield map(obj)
//         } else {
//           yield obj as unknown as TransformedElement
//         }
//       }
//     }
//   }

//   private async increment(key: Uint8Array): Promise<number> {
//     let val = await this.getCoercedOrDefault<number>(key, u8aToNumber, 0)
//     // Always store using 4 bytes, max value 2**32 - 1
//     await this.db.put(key, toU8a(val + 1, DEFAULT_SERIALIZED_NUMBER_LENGTH))
//     return val + 1
//   }

//   private async addBalance(key: Uint8Array, amount: Balance): Promise<void> {
//     let val = await this.getCoercedOrDefault<Balance>(
//       key,
//       (u) => Balance.deserialize(u, BalanceType.HOPR),
//       Balance.zero(BalanceType.HOPR)
//     )
//     await this.db.put(key, val.add(amount).serialize_value())
//   }

//   private async subBalance(key: Uint8Array, amount: Balance): Promise<void> {
//     let val = await this.getCoercedOrDefault<Balance>(
//       key,
//       (u) => Balance.deserialize(u, BalanceType.HOPR),
//       Balance.zero(BalanceType.HOPR)
//     )
//     await this.db.put(key, val.sub(amount).serialize_value())
//   }

//   /**
//    * Get unacknowledged tickets.
//    * @param filter optionally filter by signer
//    * @returns an array of all unacknowledged tickets
//    */
//   public async getUnacknowledgedTickets(filter?: { signer: Address }): Promise<UnacknowledgedTicket[]> {
//     const filterFunc = (pending: PendingAcknowledgement): boolean => {
//       if (pending.is_msg_sender() == true) {
//         return false
//       }

//       // if signer provided doesn't match our ticket's signer dont add it to the list
//       if (filter?.signer && pending.ticket().signer.eq(filter.signer)) {
//         return false
//       }
//       return true
//     }

//     return await this.getAll<PendingAcknowledgement, UnacknowledgedTicket>(
//       {
//         prefix: PENDING_ACKNOWLEDGEMENTS_PREFIX,
//         suffixLength: HalfKeyChallenge.size()
//       },
//       PendingAcknowledgement.deserialize,
//       filterFunc,
//       (pending: PendingAcknowledgement) => pending.ticket()
//     )
//   }

//   public async getPendingAcknowledgement(halfKeyChallenge: HalfKeyChallenge): Promise<PendingAcknowledgement> {
//     return await this.getCoerced<PendingAcknowledgement>(
//       createPendingAcknowledgement(halfKeyChallenge),
//       PendingAcknowledgement.deserialize
//     )
//   }

//   public async storePendingAcknowledgement(
//     halfKeyChallenge: HalfKeyChallenge,
//     isMessageSender: boolean,
//     unackTicket?: UnacknowledgedTicket
//   ): Promise<void> {
//     await this.db.put(
//       createPendingAcknowledgement(halfKeyChallenge),
//       new PendingAcknowledgement(isMessageSender, unackTicket).serialize()
//     )
//   }

//   /**
//    * Get acknowledged tickets sorted by ticket index in ascending order.
//    * @param filter optionally filter by signer
//    * @returns an array of all acknowledged tickets
//    */
//   public async getAcknowledgedTickets(filter?: {
//     signer?: Address
//     channel?: ChannelEntry
//   }): Promise<AcknowledgedTicket[]> {
//     const filterFunc = (a: AcknowledgedTicket): boolean => {
//       // if signer provided doesn't match our ticket's signer don't add it to the list
//       if (filter?.signer && !a.signer.eq(filter.signer)) {
//         return false
//       }

//       if (
//         filter?.channel &&
//         (!a.signer.eq(filter.channel.source) ||
//           !filter.channel.destination.eq(this.id.to_address()) ||
//           !a.ticket.channel_epoch.eq(filter.channel.channel_epoch))
//       ) {
//         return false
//       }

//       return true
//     }
//     // sort in ascending order by ticket index: 1,2,3,4,...
//     const sortFunc = (t1: AcknowledgedTicket, t2: AcknowledgedTicket): number => t1.ticket.index.cmp(t2.ticket.index)

//     const tickets: AcknowledgedTicket[] = []

//     for await (const ticket of this.getAllIterable<AcknowledgedTicket>(
//       {
//         prefix: ACKNOWLEDGED_TICKETS_PREFIX,
//         suffixLength: EthereumChallenge.size()
//       },
//       AcknowledgedTicket.deserialize,
//       filterFunc
//     )) {
//       tickets.push(ticket)
//     }

//     return tickets.sort(sortFunc)
//   }

//   /**
//    * Deletes all acknowledged tickets in a channel and updates
//    * neglected tickets counter.
//    * @param channel in which channel to delete tickets
//    */
//   public async deleteAcknowledgedTicketsFromChannel(channel: ChannelEntry): Promise<void> {
//     const tickets = await this.getAcknowledgedTickets({ signer: channel.source })

//     const neglectedTicketsCount = await this.getCoercedOrDefault<number>(NEGLECTED_TICKET_COUNT, u8aToNumber, 0)

//     const batch = this.db.backend.batch()

//     for (const ack of tickets) {
//       batch.del(Buffer.from(createAcknowledgedTicketKey(ack.ticket.challenge, ack.ticket.channel_epoch)))
//     }

//     // only update count if there has been a change
//     if (tickets.length > 0) {
//       batch.put(
//         Buffer.from(NEGLECTED_TICKET_COUNT),
//         // store updated number in 4 bytes
//         Buffer.from(toU8a(neglectedTicketsCount + tickets.length, 4))
//       )
//     }

//     await batch.write()
//   }

//   /**
//    * Deletes an acknowledged ticket in database
//    * @param ack acknowledged ticket
//    */
//   public async delAcknowledgedTicket(ack: AcknowledgedTicket): Promise<void> {
//     await this.db.remove(createAcknowledgedTicketKey(ack.ticket.challenge, ack.ticket.channel_epoch))
//   }

//   public async replaceUnAckWithAck(halfKeyChallenge: HalfKeyChallenge, ackTicket: AcknowledgedTicket): Promise<void> {
//     const unAcknowledgedDbKey = createPendingAcknowledgement(halfKeyChallenge)
//     const acknowledgedDbKey = createAcknowledgedTicketKey(ackTicket.ticket.challenge, ackTicket.ticket.channel_epoch)

//     const serializedTicket = ackTicket.serialize()

//     await this.db.backend
//       .batch()
//       .del(Buffer.from(unAcknowledgedDbKey.buffer, unAcknowledgedDbKey.byteOffset, unAcknowledgedDbKey.byteLength))
//       .put(
//         Buffer.from(acknowledgedDbKey.buffer, acknowledgedDbKey.byteOffset, acknowledgedDbKey.byteLength),
//         Buffer.from(serializedTicket.buffer, serializedTicket.byteOffset, serializedTicket.byteLength)
//       )
//       .write()
//   }

//   /**
//    * Get tickets, both unacknowledged and acknowledged
//    * @param filter optionally filter by signer
//    * @returns an array of signed tickets
//    */
//   public async getTickets(filter?: { signer: Address }): Promise<Ticket[]> {
//     const [unAcks, acks] = await Promise.all([
//       this.getUnacknowledgedTickets(filter),
//       this.getAcknowledgedTickets(filter)
//     ])

//     const unAckTickets = unAcks.map((o: UnacknowledgedTicket) => o.ticket)
//     const ackTickets = acks.map((o: AcknowledgedTicket) => o.ticket)
//     return [...unAckTickets, ...ackTickets]
//   }

//   /**
//    * Checks whether the given packet tag is present in the database.
//    * If not, sets the packet tag and return false, otherwise return
//    * true.
//    * @param packetTag packet tag to check for
//    * @returns a Promise that resolves to true if packet tag is present in db
//    */
//   async checkAndSetPacketTag(packetTag: Uint8Array) {
//     const packetTagKey = createPacketTagKey(packetTag)
//     let present = await this.db.has(packetTagKey)
//     console.log(`===========> Checking if packet tag ${u8aToHex(packetTagKey)} is present`, present)

//     if (!present) {
//       // TODO: what is touch? Is this correct?
//       await this.db.put(packetTagKey, new Uint8Array())
//     }

//     return present
//   }

//   async storeHashIntermediaries(channelId: Hash, intermediates: IteratedHash): Promise<void> {
//     let dbBatch = this.db.backend.batch()

//     for (let i = 0; i < intermediates.count_intermediates(); i++) {
//       let intermediate = intermediates.intermediate(i)
//       const u8aKey = createCommitmentKey(channelId, intermediate.iteration)

//       dbBatch = dbBatch.put(
//         Buffer.from(u8aKey.buffer, u8aKey.byteOffset, u8aKey.byteLength),
//         Buffer.from(
//           intermediate.intermediate,
//           intermediate.intermediate.byteOffset,
//           intermediate.intermediate.byteLength
//         )
//       )
//     }
//     await dbBatch.write()
//   }

//   async getCommitment(channelId: Hash, iteration: number) {
//     return await this.db.maybeGet(createCommitmentKey(channelId, iteration))
//   }

//   async getCurrentCommitment(channelId: Hash): Promise<Hash> {
//     return new Hash(await this.db.get(createCurrentCommitmentKey(channelId)))
//   }

//   async setCurrentCommitment(channelId: Hash, commitment: Hash): Promise<void> {
//     return this.db.put(createCurrentCommitmentKey(channelId), commitment.serialize())
//   }

//   async getCurrentTicketIndex(channelId: Hash): Promise<U256 | undefined> {
//     return await this.getCoercedOrDefault<U256>(createCurrentTicketIndexKey(channelId), U256.deserialize, undefined)
//   }

//   setCurrentTicketIndex(channelId: Hash, ticketIndex: U256): Promise<void> {
//     return this.db.put(createCurrentTicketIndexKey(channelId), ticketIndex.serialize())
//   }

//   async getLatestBlockNumber(): Promise<number> {
//     if (!(await this.db.has(LATEST_BLOCK_NUMBER_KEY))) {
//       return 0
//     }
//     return new BN(await this.db.get(LATEST_BLOCK_NUMBER_KEY)).toNumber()
//   }

//   async updateLatestBlockNumber(blockNumber: BN): Promise<void> {
//     await this.db.put(LATEST_BLOCK_NUMBER_KEY, blockNumber.toBuffer())
//   }

//   async getLatestConfirmedSnapshotOrUndefined(): Promise<Snapshot | undefined> {
//     return await this.getCoercedOrDefault<Snapshot>(LATEST_CONFIRMED_SNAPSHOT_KEY, Snapshot.deserialize, undefined)
//   }

//   // Unused
//   // async updateLatestConfirmedSnapshot(snapshot: Snapshot): Promise<void> {
//   //   await this.db.put(LATEST_CONFIRMED_SNAPSHOT_KEY, snapshot.serialize())
//   // }

//   async getChannel(channelId: Hash): Promise<ChannelEntry> {
//     return await this.getCoerced<ChannelEntry>(createChannelKey(channelId), ChannelEntry.deserialize)
//   }

//   async *getChannelsIterable(filter?: (channel: ChannelEntry) => boolean): AsyncIterable<ChannelEntry> {
//     yield* this.getAllIterable<ChannelEntry>(
//       {
//         prefix: CHANNEL_PREFIX,
//         suffixLength: Hash.size()
//       },
//       ChannelEntry.deserialize,
//       filter
//     )
//   }

//   async getChannels(filter?: (channel: ChannelEntry) => boolean): Promise<ChannelEntry[]> {
//     return this.getAll<ChannelEntry>(
//       {
//         prefix: CHANNEL_PREFIX,
//         suffixLength: Hash.size()
//       },
//       ChannelEntry.deserialize,
//       filter
//     )
//   }

//   async updateChannelAndSnapshot(channelId: Hash, channel: ChannelEntry, snapshot: Snapshot): Promise<void> {
//     const serializedChannel = channel.serialize()
//     const keyU8a = createChannelKey(channelId)

//     const serializedSnapshot = snapshot.serialize()

//     await this.db.backend
//       .batch()
//       .put(
//         Buffer.from(keyU8a.buffer, keyU8a.byteOffset, keyU8a.byteLength),
//         Buffer.from(serializedChannel.buffer, serializedChannel.byteOffset, serializedChannel.byteLength)
//       )
//       .put(
//         Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY),
//         Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
//       )
//       .write()
//   }

//   async getAccount(address: Address): Promise<AccountEntry | undefined> {
//     return await this.getCoercedOrDefault<AccountEntry>(createAccountKey(address), AccountEntry.deserialize, undefined)
//   }

//   async updateAccountAndSnapshot(account: AccountEntry, snapshot: Snapshot): Promise<void> {
//     const serializedAccount = account.serialize()
//     const serializedSnapshot = snapshot.serialize()

//     await this.db.backend
//       .batch()
//       .put(
//         Buffer.from(createAccountKey(account.get_address())),
//         Buffer.from(serializedAccount.buffer, serializedAccount.byteOffset, serializedAccount.byteLength)
//       )
//       .put(
//         Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY),
//         Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
//       )
//       .write()
//   }

//   async getAccounts(filter?: (account: AccountEntry) => boolean) {
//     return this.getAll<AccountEntry>(
//       {
//         prefix: ACCOUNT_PREFIX,
//         suffixLength: Address.size()
//       },
//       AccountEntry.deserialize,
//       filter
//     )
//   }

//   async *getAccountsIterable(filter?: (account: AccountEntry) => boolean) {
//     yield* this.getAllIterable<AccountEntry>(
//       {
//         prefix: ACCOUNT_PREFIX,
//         suffixLength: Address.size()
//       },
//       AccountEntry.deserialize,
//       filter
//     )
//   }

//   public async getRedeemedTicketsValue(): Promise<Balance> {
//     return await this.getCoercedOrDefault<Balance>(
//       REDEEMED_TICKETS_VALUE,
//       (u) => Balance.deserialize(u, BalanceType.HOPR),
//       Balance.zero(BalanceType.HOPR)
//     )
//   }

//   public async getRedeemedTicketsCount(): Promise<number> {
//     return this.getCoercedOrDefault<number>(REDEEMED_TICKETS_COUNT, u8aToNumber, 0)
//   }

//   public async getNeglectedTicketsCount(): Promise<number> {
//     return this.getCoercedOrDefault<number>(NEGLECTED_TICKET_COUNT, u8aToNumber, 0)
//   }

//   public async getPendingTicketCount(): Promise<number> {
//     return (await this.getUnacknowledgedTickets()).length
//   }

//   public async getPendingBalanceTo(counterparty: Address): Promise<Balance> {
//     return await this.getCoercedOrDefault<Balance>(
//       createPendingTicketsCountKey(counterparty),
//       (u) => Balance.deserialize(u, BalanceType.HOPR),
//       Balance.zero(BalanceType.HOPR)
//     )
//   }

//   public async getLosingTicketCount(): Promise<number> {
//     return await this.getCoercedOrDefault<number>(LOSING_TICKET_COUNT, u8aToNumber, 0)
//   }

//   public async markPending(ticket: Ticket) {
//     return await this.addBalance(createPendingTicketsCountKey(ticket.counterparty), ticket.amount)
//   }

//   public async resolvePending(ticket: Partial<Ticket>, snapshot: Snapshot) {
//     let val = await this.getCoercedOrDefault<Balance>(
//       createPendingTicketsCountKey(ticket.counterparty),
//       (u) => Balance.deserialize(u, BalanceType.HOPR),
//       Balance.zero(BalanceType.HOPR)
//     )

//     const serializedSnapshot = snapshot.serialize()
//     const u8aPendingKey = createPendingTicketsCountKey(ticket.counterparty)

//     await this.db.backend
//       .batch()
//       .put(
//         Buffer.from(u8aPendingKey.buffer, u8aPendingKey.byteOffset, u8aPendingKey.byteLength),
//         Buffer.from(val.sub(val).serialize_value())
//       )
//       .put(
//         Buffer.from(
//           LATEST_CONFIRMED_SNAPSHOT_KEY.buffer,
//           LATEST_CONFIRMED_SNAPSHOT_KEY.byteOffset,
//           LATEST_CONFIRMED_SNAPSHOT_KEY.byteLength
//         ),
//         Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
//       )
//       .write()
//   }

//   public async markRedeemeed(a: AcknowledgedTicket): Promise<void> {
//     await this.increment(REDEEMED_TICKETS_COUNT)
//     await this.delAcknowledgedTicket(a)
//     await this.addBalance(REDEEMED_TICKETS_VALUE, a.ticket.amount)
//     await this.subBalance(createPendingTicketsCountKey(a.ticket.counterparty), a.ticket.amount)
//   }

//   public async markLosingAckedTicket(a: AcknowledgedTicket): Promise<void> {
//     await this.increment(LOSING_TICKET_COUNT)
//     await this.delAcknowledgedTicket(a)
//     await this.subBalance(createPendingTicketsCountKey(a.ticket.counterparty), a.ticket.amount)
//   }

//   public async getRejectedTicketsValue(): Promise<Balance> {
//     return await this.getCoercedOrDefault<Balance>(
//       REJECTED_TICKETS_VALUE,
//       (u) => Balance.deserialize(u, BalanceType.HOPR),
//       Balance.zero(BalanceType.HOPR)
//     )
//   }

//   public async getRejectedTicketsCount(): Promise<number> {
//     return this.getCoercedOrDefault<number>(REJECTED_TICKETS_COUNT, u8aToNumber, 0)
//   }

//   public async markRejected(t: Ticket): Promise<void> {
//     await this.increment(REJECTED_TICKETS_COUNT)
//     await this.addBalance(REJECTED_TICKETS_VALUE, t.amount)
//   }

//   public async getChannelX(src: Address, dest: Address): Promise<ChannelEntry> {
//     return await this.getChannel(generate_channel_id(src, dest))
//   }

//   public async getChannelTo(dest: Address): Promise<ChannelEntry> {
//     return await this.getChannel(generate_channel_id(this.id.to_address(), dest))
//   }

//   public async getChannelFrom(src: Address): Promise<ChannelEntry> {
//     return await this.getChannel(generate_channel_id(src, this.id.to_address()))
//   }

//   public async getChannelsFrom(address: Address) {
//     return this.getChannels((channel) => {
//       return address.eq(channel.source)
//     })
//   }

//   public async *getChannelsFromIterable(address: Address) {
//     for await (const channel of this.getChannelsIterable()) {
//       if (address.eq(channel.source)) {
//         yield channel
//       }
//     }
//   }

//   public async getChannelsTo(address: Address) {
//     return this.getChannels((channel) => {
//       return address.eq(channel.destination)
//     })
//   }

//   public async *getChannelsToIterable(address: Address) {
//     for await (const channel of this.getChannelsIterable()) {
//       if (address.eq(channel.destination)) {
//         yield channel
//       }
//     }
//   }

//   public async getHoprBalance(): Promise<Balance> {
//     return this.getCoercedOrDefault<Balance>(
//       HOPR_BALANCE_KEY,
//       (u) => Balance.deserialize(u, BalanceType.HOPR),
//       Balance.zero(BalanceType.HOPR)
//     )
//   }

//   public async setHoprBalance(value: Balance): Promise<void> {
//     return this.db.put(HOPR_BALANCE_KEY, value.serialize_value())
//   }

//   public async addHoprBalance(value: Balance, snapshot: Snapshot): Promise<void> {
//     const val = await this.getCoercedOrDefault<Balance>(
//       HOPR_BALANCE_KEY,
//       (u) => Balance.deserialize(u, BalanceType.HOPR),
//       Balance.zero(BalanceType.HOPR)
//     )

//     const serializedSnapshot = snapshot.serialize()

//     await this.db.backend
//       .batch()
//       .put(
//         Buffer.from(HOPR_BALANCE_KEY.buffer, HOPR_BALANCE_KEY.byteOffset, HOPR_BALANCE_KEY.byteLength),
//         Buffer.from(val.add(value).serialize_value())
//       )
//       .put(
//         Buffer.from(
//           LATEST_CONFIRMED_SNAPSHOT_KEY.buffer,
//           LATEST_CONFIRMED_SNAPSHOT_KEY.byteOffset,
//           LATEST_CONFIRMED_SNAPSHOT_KEY.byteLength
//         ),
//         Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
//       )
//       .write()
//   }

//   public async subHoprBalance(value: Balance, snapshot: Snapshot): Promise<void> {
//     const val = await this.getCoercedOrDefault<Balance>(
//       HOPR_BALANCE_KEY,
//       (u) => Balance.deserialize(u, BalanceType.HOPR),
//       Balance.zero(BalanceType.HOPR)
//     )

//     const serializedSnapshot = snapshot.serialize()

//     await this.db.backend
//       .batch()
//       .put(
//         Buffer.from(HOPR_BALANCE_KEY.buffer, HOPR_BALANCE_KEY.byteOffset, HOPR_BALANCE_KEY.byteLength),
//         Buffer.from(val.sub(value).serialize_value())
//       )
//       .put(
//         Buffer.from(
//           LATEST_CONFIRMED_SNAPSHOT_KEY.buffer,
//           LATEST_CONFIRMED_SNAPSHOT_KEY.byteOffset,
//           LATEST_CONFIRMED_SNAPSHOT_KEY.byteLength
//         ),
//         Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
//       )
//       .write()
//   }

//   static serializeArrayOfPubKeys(pKeys: PublicKey[]): Uint8Array {
//     return u8aConcat(...pKeys.map((p) => p.serialize(true)))
//   }

//   static deserializeArrayOfPubKeys(arr: Uint8Array): PublicKey[] {
//     const result: PublicKey[] = []
//     let SIZE_PUBKEY_COMPRESSED = PublicKey.size_compressed()
//     let SIZE_PUBKEY_UNCOMPRESSED = PublicKey.size_uncompressed()
//     for (let offset = 0; offset < arr.length; ) {
//       switch (arr[offset] as Secp256k1PublicKeyPrefix) {
//         case Secp256k1PublicKeyPrefix.COMPRESSED_NEGATIVE:
//         case Secp256k1PublicKeyPrefix.COMPRESSED_POSITIVE:
//           if (arr.length < offset + SIZE_PUBKEY_COMPRESSED) {
//             throw Error(`Invalid array length. U8a has ${offset + SIZE_PUBKEY_COMPRESSED - arr.length} to few elements`)
//           }
//           // clone array
//           result.push(PublicKey.deserialize(arr.slice(offset, offset + SIZE_PUBKEY_COMPRESSED)))
//           offset += SIZE_PUBKEY_COMPRESSED
//           break
//         case Secp256k1PublicKeyPrefix.UNCOMPRESSED:
//           if (arr.length < offset + SIZE_PUBKEY_UNCOMPRESSED) {
//             throw Error(
//               `Invalid array length. U8a has ${offset + SIZE_PUBKEY_UNCOMPRESSED - arr.length} to few elements`
//             )
//           }
//           // clone array
//           result.push(PublicKey.deserialize(arr.slice(offset, offset + SIZE_PUBKEY_UNCOMPRESSED)))
//           offset += SIZE_PUBKEY_UNCOMPRESSED
//           break
//         default:
//           throw Error(`Invalid prefix ${u8aToHex(arr.subarray(offset, offset + 1))} at ${offset}`)
//       }
//     }

//     return result
//   }

//   /**
//    * Hopr Network Registry
//    * Link hoprNode to an ETH address.
//    * @param pubKey the node to register
//    * @param account the account that made the transaction
//    * @param snapshot
//    */
//   public async addToNetworkRegistry(pubKey: PublicKey, account: Address, snapshot: Snapshot): Promise<void> {
//     let registeredNodes: PublicKey[] = []
//     try {
//       registeredNodes = await this.findHoprNodesUsingAccountInNetworkRegistry(account)
//     } catch (error) {}

//     const serializedSnapshot = snapshot.serialize()

//     // Prevents from adding nodes more than once
//     for (const registeredNode of registeredNodes) {
//       if (registeredNode.eq(pubKey)) {
//         // update snapshot
//         await this.db.put(
//           Buffer.from(
//             LATEST_CONFIRMED_SNAPSHOT_KEY.buffer,
//             LATEST_CONFIRMED_SNAPSHOT_KEY.byteOffset,
//             LATEST_BLOCK_NUMBER_KEY.byteLength
//           ),
//           Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
//         )
//         // already registered, nothing to do
//         return
//       }
//     }

//     // add new node to the list
//     registeredNodes.push(pubKey)

//     const serializedRegisteredNodes = HoprDB.serializeArrayOfPubKeys(registeredNodes)
//     const serializedAccount = account.serialize()

//     await this.db.backend
//       .batch()
//       // node public key to address (M->1)
//       .put(
//         Buffer.from(createNetworkRegistryEntryKey(pubKey)),
//         Buffer.from(serializedAccount.buffer, serializedAccount.byteOffset, serializedAccount.byteLength)
//       )
//       // address to node public keys (1->M) in the format of key -> PublicKey[]
//       .put(
//         Buffer.from(createNetworkRegistryAddressToPublicKeyKey(account)),
//         Buffer.from(
//           serializedRegisteredNodes.buffer,
//           serializedRegisteredNodes.byteOffset,
//           serializedRegisteredNodes.byteLength
//         )
//       )
//       .put(
//         Buffer.from(LATEST_CONFIRMED_SNAPSHOT_KEY),
//         Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
//       )
//       .write()
//   }

//   /**
//    * Do a reverse find by searching the stored account to return
//    * the associated public keys of registered HOPR nodes.
//    * @param account
//    * @returns array of PublicKey of the associated HOPR nodes
//    */
//   public async findHoprNodesUsingAccountInNetworkRegistry(account: Address): Promise<PublicKey[]> {
//     const pubKeys = await this.getCoercedOrDefault<PublicKey[]>(
//       createNetworkRegistryAddressToPublicKeyKey(account),
//       HoprDB.deserializeArrayOfPubKeys,
//       undefined
//     )

//     if (!pubKeys) {
//       throw Error('HoprNode not found')
//     }

//     return pubKeys
//   }

//   /**
//    * Hopr Network Registry
//    * Unlink hoprNode to an ETH address by removing the entry.
//    * @param pubKey the node's x
//    * @param account the account to use so we can search for the key in the database
//    * @param snapshot
//    */
//   public async removeFromNetworkRegistry(pubKey: PublicKey, account: Address, snapshot: Snapshot): Promise<void> {
//     let registeredNodes: PublicKey[] = []
//     try {
//       registeredNodes = await this.findHoprNodesUsingAccountInNetworkRegistry(account)
//     } catch (error) {
//       log(`cannot remove node from network registry due to ${error}`)
//       throw Error('HoprNode not registered to the account')
//     }

//     // Remove all occurences, even if there are more than one
//     registeredNodes = registeredNodes.filter((registeredNode: PublicKey) => !registeredNode.eq(pubKey))

//     const entryKey = createNetworkRegistryEntryKey(pubKey)

//     const serializedRegisteredNodes = HoprDB.serializeArrayOfPubKeys(registeredNodes)
//     const serializedSnapshot = snapshot.serialize()

//     await this.db.backend
//       .batch()
//       .del(Buffer.from(entryKey.buffer, entryKey.byteOffset, entryKey.byteLength))
//       // address to node public keys (1->M) in the format of key -> PublicKey[]
//       .put(
//         Buffer.from(createNetworkRegistryAddressToPublicKeyKey(account)),
//         Buffer.from(
//           serializedRegisteredNodes.buffer,
//           serializedRegisteredNodes.byteOffset,
//           serializedRegisteredNodes.byteLength
//         )
//       )
//       .put(
//         Buffer.from(
//           LATEST_CONFIRMED_SNAPSHOT_KEY.buffer,
//           LATEST_CONFIRMED_SNAPSHOT_KEY.byteOffset,
//           LATEST_CONFIRMED_SNAPSHOT_KEY.byteLength
//         ),
//         Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
//       )
//       .write()
//   }

//   /**
//    * Hopr Network Registry
//    * Get address associated with hoprNode.
//    * @param hoprNode the node to register
//    * @returns ETH address
//    */
//   public async getAccountFromNetworkRegistry(hoprNode: PublicKey): Promise<Address> {
//     return this.getCoerced<Address>(createNetworkRegistryEntryKey(hoprNode), Address.deserialize)
//   }

//   /**
//    * Hopr Network Registry
//    * Set address as eligible.
//    * @param account the account that made the transaction
//    * @param snapshot
//    */
//   public async setEligible(account: Address, eligible: boolean, snapshot: Snapshot): Promise<void> {
//     const key = Buffer.from(createNetworkRegistryAddressEligibleKey(account))

//     const serializedSnapshot = snapshot.serialize()

//     if (eligible) {
//       await this.db.backend
//         .batch()
//         .put(key, Buffer.from([]))
//         .put(
//           Buffer.from(
//             LATEST_CONFIRMED_SNAPSHOT_KEY.buffer,
//             LATEST_CONFIRMED_SNAPSHOT_KEY.byteOffset,
//             LATEST_CONFIRMED_SNAPSHOT_KEY.byteLength
//           ),
//           Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
//         )
//         .write()
//     } else {
//       await this.db.backend
//         .batch()
//         .del(key)
//         .put(
//           Buffer.from(
//             LATEST_CONFIRMED_SNAPSHOT_KEY.buffer,
//             LATEST_CONFIRMED_SNAPSHOT_KEY.byteOffset,
//             LATEST_CONFIRMED_SNAPSHOT_KEY.byteLength
//           ),
//           Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
//         )
//         .write()
//     }
//   }

//   /**
//    * Hopr Network Registry
//    * @param account the account that made the transaction
//    * @returns true if account is eligible
//    */
//   public async isEligible(account: Address): Promise<boolean> {
//     return this.getCoercedOrDefault<boolean>(createNetworkRegistryAddressEligibleKey(account), () => true, false)
//   }

//   /**
//    * Hopr Network Registry
//    * @param enabled whether register is enabled
//    */
//   public async setNetworkRegistryEnabled(enabled: boolean, snapshot: Snapshot): Promise<void> {
//     const serializedSnapshot = snapshot.serialize()

//     await this.db.backend
//       .batch()
//       .put(Buffer.from(NETWORK_REGISTRY_ENABLED_PREFIX), Buffer.from([enabled ? 1 : 0]))
//       .put(
//         Buffer.from(
//           LATEST_CONFIRMED_SNAPSHOT_KEY.buffer,
//           LATEST_CONFIRMED_SNAPSHOT_KEY.byteOffset,
//           LATEST_CONFIRMED_SNAPSHOT_KEY.byteLength
//         ),
//         Buffer.from(serializedSnapshot.buffer, serializedSnapshot.byteOffset, serializedSnapshot.byteLength)
//       )
//       .write()
//   }

//   /**
//    * Check ifs Network registry is enabled
//    * @returns true if register is enabled or if key is not preset in the dababase
//    */
//   public async isNetworkRegistryEnabled(): Promise<boolean> {
//     return this.getCoercedOrDefault<boolean>(NETWORK_REGISTRY_ENABLED_PREFIX, (v) => v[0] != 0, true)
//   }

//   static createMock(id?: PublicKey): HoprDB {
//     const mock: HoprDB = {
//       id:
//         id ?? PublicKey.from_privkey(stringToU8a('0x1464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8')),
//       // CommonJS / ESM issue
//       // @ts-ignore
//       db: new LevelDb()
//     } as any
//     Object.setPrototypeOf(mock, HoprDB.prototype)

//     return mock
//   }

//   // Stores an already serialized object in the database.
//   // @param namespace Namespace which is used for the object.
//   // @param key Key to identify the object in the namespace.
//   // @param object Serialized object.
//   public async putSerializedObject(namespace: string, key: string, object: Uint8Array): Promise<void> {
//     return await this.db.put(createObjectKey(namespace, key), object)
//   }

//   // Reads an a serialized object from the database.
//   // @param namespace Namespace which is used for the object.
//   // @param key Key to identify the object in the namespace.
//   // @returns The serialized object or `undefined` if none was found.
//   public async getSerializedObject(namespace: string, key: string): Promise<Uint8Array | undefined> {
//     return await this.db.maybeGet(createObjectKey(namespace, key))
//   }

//   // Deletes an object from the database. Silently succeeds if object wasn't in
//   // the database.
//   // @param namespace Namespace which is used for the object.
//   // @param key Key to identify the object in the namespace.
//   public async deleteObject(namespace: string, key: string): Promise<void> {
//     await this.db.remove(createObjectKey(namespace, key))
//   }
// }
