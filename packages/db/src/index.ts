import { existsSync, mkdirSync } from 'fs'
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

export default class HoprDB {
  private db: LevelUp

  constructor(options: {
      id? : string,
      bootstrapNode?: boolean,
      createDbIfNotExist?: boolean,
      dbPath?: string,
      db?: LevelUp
    }
  ){
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

  public async close(){
    return this.db.close()
  }

  public async getIdentity(){
    return await this.db.get(Buffer.from(KeyPair))
  }

  public async storeIdentity(id: Buffer){
    await this.db.put(Buffer.from(KeyPair), id)
  }

  public async getUnacknowledgedTicketsStream(){
    return this.db
      .createReadStream({
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
}

