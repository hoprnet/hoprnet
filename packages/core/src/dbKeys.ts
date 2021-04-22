import type { LevelUp } from 'levelup'
import { Hash, Acknowledgement, UnacknowledgedTicket } from '@hoprnet/hopr-core-ethereum'
import { u8aAdd, toU8a } from '@hoprnet/hopr-utils'
import debug from 'debug'
const log = debug('hopr-core:db')

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

export async function getUnacknowledgedTickets(db: LevelUp, key: Hash): Promise<UnacknowledgedTicket | undefined> {
  const unAcknowledgedDbKey = UnAcknowledgedTickets(key.serialize())
  try {
    const buff = await db.get(Buffer.from(unAcknowledgedDbKey))
    if (buff.length === 0) {
      return undefined
    }
    return UnacknowledgedTicket.deserialize(buff)
  } catch (err) {
    if (err.notFound) {
      return undefined
    }
    throw err
  }
}

export async function deleteTicket(db: LevelUp, key: Hash) {
  const k = UnAcknowledgedTickets(key.serialize())
  await db.del(Buffer.from(k))
}

export async function incrementTicketCounter(db: LevelUp): Promise<Uint8Array> {
  let ticketCounter
  try {
    let tmpTicketCounter = await db.get(Buffer.from(AcknowledgedTicketCounter()))
    ticketCounter = u8aAdd(true, tmpTicketCounter, toU8a(1, ACKNOWLEDGED_TICKET_INDEX_LENGTH))
  } catch (err) {
    // Set ticketCounter to initial value
    ticketCounter = toU8a(0, ACKNOWLEDGED_TICKET_INDEX_LENGTH)
  }
  return ticketCounter
}

export async function replaceTicketWithAcknowledgement(db: LevelUp, key: Hash, acknowledgment: Acknowledgement) {
  const ticketCounter = await incrementTicketCounter(db)
  const unAcknowledgedDbKey = UnAcknowledgedTickets(key.serialize())
  const acknowledgedDbKey = AcknowledgedTickets(ticketCounter)
  try {
    await db
      .batch()
      .del(Buffer.from(unAcknowledgedDbKey))
      .put(Buffer.from(acknowledgedDbKey), Buffer.from(acknowledgment.serialize()))
      .put(Buffer.from(AcknowledgedTicketCounter()), Buffer.from(ticketCounter))
      .write()
  } catch (err) {
    log(`ERROR: Error while writing to database. Error was ${err.message}.`)
  }
}

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

/**
 * Checks if the given packet tag is present in the database
 * @param db database to store tags
 * @param tag packet tag
 * @returns true if tag is unknown, otherwise false
 */
export async function checkPacketTag(db: LevelUp, tag: Uint8Array): Promise<boolean> {
  let tagPresent = true

  const dbKey = PacketTag(tag)
  try {
    await db.get(dbKey)
  } catch (err) {
    if (err.notFound) {
      tagPresent = false
    }
  }

  if (!tagPresent) {
    await db.put(dbKey, Buffer.from(''))
  }

  return !tagPresent
}

function PacketTag(tag: Uint8Array): Uint8Array {
  return allocationHelper([
    [PACKET_PREFIX.length, PACKET_PREFIX],
    [packetTagSubPrefix.length, packetTagSubPrefix],
    [SEPERATOR.length, SEPERATOR],
    [tag.length, tag]
  ])
}

export const KeyPair = encoder.encode('keyPair')

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
