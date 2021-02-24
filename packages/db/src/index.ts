import { existsSync, mkdirSync } from 'fs'
import { u8aToNumber } from '@hoprnet/hopr-utils'
import path from 'path'
import levelup, { LevelUp } from 'levelup'
import leveldown from 'leveldown'
import Debug from 'debug'

const log = Debug('hopr-db')

const VERSION = 'TODO' // TODO

const encoder = new TextEncoder()
const TICKET_PREFIX: Uint8Array = encoder.encode('tickets-')
const PACKET_PREFIX: Uint8Array = encoder.encode('packets-')
const SEPERATOR: Uint8Array = encoder.encode('-')

const acknowledgedSubPrefix = encoder.encode('acknowledged-')
const acknowledgedTicketCounter = encoder.encode('acknowledgedCounter')

const unAcknowledgedSubPrefix = encoder.encode('unacknowledged-')

const packetTagSubPrefix = encoder.encode('tag-')

const KEY_LENGTH = 32

export const ACKNOWLEDGED_TICKET_INDEX_LENGTH = 8

export function AcknowledgedTickets(index: Uint8Array): Uint8Array {
  return allocationHelper([
    [TICKET_PREFIX.length, TICKET_PREFIX],
    [acknowledgedSubPrefix.length, acknowledgedSubPrefix],
    [ACKNOWLEDGED_TICKET_INDEX_LENGTH, index]
  ])
}

export function AcknowledgedTicketsParse(arr: Uint8Array): Uint8Array {
  return arr.slice(TICKET_PREFIX.length + acknowledgedSubPrefix.length, arr.length)
}

export function AcknowledgedTicketCounter() {
  return allocationHelper([
    [TICKET_PREFIX.length, TICKET_PREFIX],
    [acknowledgedTicketCounter.length, acknowledgedTicketCounter]
  ])
}

export function UnAcknowledgedTickets(hashedKey: Uint8Array): Uint8Array {
  return allocationHelper([
    [TICKET_PREFIX.length, TICKET_PREFIX],
    [unAcknowledgedSubPrefix.length, unAcknowledgedSubPrefix],
    [SEPERATOR.length, SEPERATOR],
    [KEY_LENGTH, hashedKey]
  ])
}

export function UnAcknowledgedTicketsParse(arg: Uint8Array): Uint8Array {
  return arg.slice(
    TICKET_PREFIX.length + unAcknowledgedSubPrefix.length + SEPERATOR.length,
    TICKET_PREFIX.length + unAcknowledgedSubPrefix.length + SEPERATOR.length + KEY_LENGTH
  )
}

export function PacketTag(tag: Uint8Array): Uint8Array {
  return allocationHelper([
    [PACKET_PREFIX.length, PACKET_PREFIX],
    [packetTagSubPrefix.length, packetTagSubPrefix],
    [SEPERATOR.length, SEPERATOR],
    [tag.length, tag]
  ])
}

type Config = [number, Uint8Array]

function allocationHelper(arr: Config[]) {
  const totalLength = arr.reduce((acc, current) => acc + current[0], 0)

  let result = new Uint8Array(totalLength)

  let offset = 0
  for (let [size, data] of arr) {
    result.set(data, offset)
    offset += size
  }

  return result
}
const KeyPair = encoder.encode('keyPair')

const defaultDBPath = (id: string | number, isBootstrap: boolean): string => {
  let folder: string
  if (isBootstrap) {
    folder = `bootstrap`
  } else if (id) {
    folder = `node_${id}`
  } else {
    folder = `node`
  }
  return path.join(process.cwd(), 'db', VERSION, folder)
}


const PREFIX = encoder.encode('payments-')
const channelSubPrefix = encoder.encode('channel-')
const channelEntrySubPrefix = encoder.encode('channelEntry-')
const challengeSubPrefix = encoder.encode('challenge-')
const channelIdSubPrefix = encoder.encode('channelId-')
const nonceSubPrefix = encoder.encode('nonce-')
const ticketSubPrefix = encoder.encode('tickets-')
const onChainSecretIntermediary = encoder.encode('onChainSecretIntermediary-')
const latestBlockNumber = encoder.encode('latestBlockNumber')
const latestConfirmedSnapshot = encoder.encode('latestConfirmedSnapshot')

const ON_CHAIN_SECRET_ITERATION_WIDTH = 4 // bytes

/**
 * Returns the db-key under which the channel is saved.
 * @param counterparty counterparty of the channel
 */
export function Channel(counterparty: Uint8Array): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [channelSubPrefix.length, channelSubPrefix],
    [counterparty.length, counterparty]
  ])
}

/**
 * Reconstructs the channelId from a db-key.
 * @param arr a channel db-key
 */
export function ChannelKeyParse(arr: Uint8Array): Uint8Array {
  return arr.slice(PREFIX.length + channelSubPrefix.length)
}

/**
 * Returns the db-key under which the latest known block number is saved in the database.
 */
export function LatestBlockNumber(): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [latestBlockNumber.length, latestBlockNumber]
  ])
}

/**
 * Returns the db-key under which the latest confirmed snapshot is saved in the database.
 */
export function LatestConfirmedSnapshot(): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [latestConfirmedSnapshot.length, latestConfirmedSnapshot]
  ])
}

/**
 * Returns the db-key under which channel entries are saved.
 * @param partyA the accountId of partyA
 * @param partyB the accountId of partyB
 */
export function ChannelEntry(partyA: Uint8Array, partyB: Uint8Array): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [channelEntrySubPrefix.length, channelEntrySubPrefix],
    [Public.SIZE, partyA],
    [SEPERATOR.length, SEPERATOR],
    [Public.SIZE, partyB]
  ])
}

/**
 * Reconstructs parties from a channel entry db-key.
 * @param arr a challenge db-key
 * @returns an array containing partyA's and partyB's accountIds
 */
export function ChannelEntryParse(arr: Uint8Array): [Public, Public] {
  const partyAStart = PREFIX.length + channelEntrySubPrefix.length
  const partyAEnd = partyAStart + Public.SIZE
  const partyBStart = partyAEnd + SEPERATOR.length
  const partyBEnd = partyBStart + Public.SIZE

  return [new Public(arr.slice(partyAStart, partyAEnd)), new Public(arr.slice(partyBStart, partyBEnd))]
}

/**
 * Returns the db-key under which the challenge is saved.
 * @param channelId channelId of the channel
 * @param challenge challenge to save
 */
export function Challenge(channelId: Uint8Array, challenge: Uint8Array): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [challengeSubPrefix.length, challengeSubPrefix],
    [Hash.SIZE, channelId],
    [SEPERATOR.length, SEPERATOR],
    [Hash.SIZE, challenge]
  ])
}

/**
 * Reconstructs channelId and the specified challenge from a challenge db-key.
 * @param arr a challenge db-key
 */
export function ChallengeKeyParse(arr: Uint8Array): [Hash, Hash] {
  const channelIdStart = PREFIX.length + challengeSubPrefix.length
  const channelIdEnd = channelIdStart + Hash.SIZE
  const challengeStart = channelIdEnd + SEPERATOR.length
  const challengeEnd = challengeStart + Hash.SIZE

  return [new Hash(arr.slice(channelIdStart, channelIdEnd)), new Hash(arr.slice(challengeStart, challengeEnd))]
}

/**
 * Returns the db-key under which signatures of acknowledgements are saved.
 * @param signatureHash hash of an ackowledgement signature
 */
export function ChannelId(signatureHash: Types.Hash): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [channelIdSubPrefix.length, channelIdSubPrefix],
    [Hash.SIZE, signatureHash]
  ])
}

/**
 * Returns the db-key under which nonces are saved.
 * @param channelId channelId of the channel
 * @param nonce the nonce
 */
export function Nonce(channelId: Types.Hash, nonce: Types.Hash): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [nonceSubPrefix.length, nonceSubPrefix],
    [Hash.SIZE, channelId],
    [SEPERATOR.length, SEPERATOR],
    [Hash.SIZE, nonce]
  ])
}

export function OnChainSecret(): Uint8Array {
  return OnChainSecretIntermediary(0)
}

export function OnChainSecretIntermediary(iteration: number): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [onChainSecretIntermediary.length, onChainSecretIntermediary],
    [SEPERATOR.length, SEPERATOR],
    [ON_CHAIN_SECRET_ITERATION_WIDTH, toU8a(iteration, ON_CHAIN_SECRET_ITERATION_WIDTH)]
  ])
}

/**
 * Returns the db-key under which the tickets are saved in the database.
 */
export function AcknowledgedTicket(counterPartyPubKey: Types.Public, challange: Types.Hash): Uint8Array {
  return allocationHelper([
    [ticketSubPrefix.length, ticketSubPrefix],
    [acknowledgedSubPrefix.length, acknowledgedSubPrefix],
    [counterPartyPubKey.length, counterPartyPubKey],
    [SEPERATOR.length, SEPERATOR],
    [challange.length, challange]
  ])
}

/**
 * Reconstructs counterPartyPubKey and the specified challenge from a AcknowledgedTicket db-key.
 * @param arr a AcknowledgedTicket db-key
 * @param props additional arguments
 */
export function AcknowledgedTicketParse(arr: Uint8Array): [Public, Hash] {
  const counterPartyPubKeyStart = ticketSubPrefix.length + acknowledgedSubPrefix.length
  const counterPartyPubKeyEnd = counterPartyPubKeyStart + Public.SIZE
  const challengeStart = counterPartyPubKeyEnd + SEPERATOR.length
  const challengeEnd = challengeStart + Hash.SIZE

  return [
    new Public(arr.slice(counterPartyPubKeyStart, counterPartyPubKeyEnd)),
    new Hash(arr.slice(challengeStart, challengeEnd))
  ]
}


export default class HoprDB {
  private db: LevelUp

  constructor(options: {
    id?: string
    bootstrapNode?: boolean
    createDbIfNotExist?: boolean
    dbPath?: string
    db?: LevelUp
  }) {
    if (options.db) {
      this.db = options.db
      return
    }

    let dbPath: string
    if (options.dbPath) {
      dbPath = options.dbPath
    } else {
      dbPath = defaultDBPath(options.id, options.bootstrapNode)
    }

    dbPath = path.resolve(dbPath)

    log('using db at ', dbPath)
    if (!existsSync(dbPath)) {
      log('db does not exist, creating?:', options.createDbIfNotExist)
      if (options.createDbIfNotExist) {
        mkdirSync(dbPath, { recursive: true })
      } else {
        throw new Error('Database does not exist: ' + dbPath)
      }
    }

    this.db = levelup(leveldown(dbPath))
  }

  public async close() {
    return this.db.close()
  }

  public async getIdentity() {
    return await this.db.get(Buffer.from(KeyPair))
  }

  public async storeIdentity(id: Buffer) {
    await this.db.put(Buffer.from(KeyPair), id)
  }

  public async getUnacknowledgedTicketsStream() {
    return this.db.createReadStream({
      gte: Buffer.from(UnAcknowledgedTickets(new Uint8Array(0x00)))
    })
  }

  public async deleteUnacknowledgedTickets(ids) {
    await this.db.batch(
      await Promise.all(
        ids.map(async (id) => {
          return {
            type: 'del',
            key: Buffer.from(UnAcknowledgedTickets(id))
          }
        })
      )
    )
  }

  public async getAcknowledgedTicketsStream() {
    return this.db.createReadStream({
      gte: Buffer.from(AcknowledgedTickets(new Uint8Array(0x00)))
    })
  }

  /**
   * Delete acknowledged ticket in database
   * @param index Uint8Array
   */
  public async deleteAcknowledgedTicket(index: Uint8Array): Promise<void> {
    await this.db.del(Buffer.from(AcknowledgedTickets(index)))
  }

  public async checkPacketSeen(tag){
    const key = PacketTag(tag)
    try {
      await this.db.get(key)
    } catch (err) {
      if (err.type === 'NotFoundError' || err.notFound === undefined || !err.notFound) {
        await this.db.put(Buffer.from(key), Buffer.from(''))
        return false
      } else {
        throw err
      }
    }

    throw Error('Key is already present. Cannot accept packet because it might be a duplicate.')
  }

  public async storeUnacknowledgedTicket(k, v){
    return await this.db.put(
      Buffer.from(UnAcknowledgedTickets(k)),
      Buffer.from(v)
    )
  }

  /**
   * Queries the database to find the latest known block number.
   * @param connector
   * @returns promise that resolves to a number
   */
  public async getLatestBlockNumber(): Promise<number> {
    try {
      return u8aToNumber(await this.db.get(Buffer.from(LatestBlockNumber())) as number
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
  await db.put(Buffer.from(LATEST_BLOCK_KEY), blockNumber.toBuffer())
}
}
