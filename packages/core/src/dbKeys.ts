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
