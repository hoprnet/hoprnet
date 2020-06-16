import { pubKeyToPeerId } from './utils'
import type PeerId from 'peer-id'

const encoder = new TextEncoder()
const TICKET_PREFIX: Uint8Array = encoder.encode('tickets-')
const PACKET_PREFIX: Uint8Array = encoder.encode('packets-')
const SEPERATOR: Uint8Array = encoder.encode('-')

const acknowledgedSubPrefix = encoder.encode('acknowledged-')
const unAcknowledgedSubPrefix = encoder.encode('unacknowledged-')

const packetTagSubPrefix = encoder.encode('tag-')

const COMPRESSED_PUBLIC_KEY_LENGTH = 33
const KEY_LENGTH = 32

export function AcknowledgedTickets(publicKeyCounterparty: Uint8Array, id: Uint8Array): Uint8Array {
  return allocationHelper([
    [TICKET_PREFIX.length, TICKET_PREFIX],
    [acknowledgedSubPrefix.length, acknowledgedSubPrefix],
    [publicKeyCounterparty.length, publicKeyCounterparty],
    [SEPERATOR.length, SEPERATOR],
    [id.length, id],
  ])
}

export function UnAcknowledgedTickets(publicKeyCounterparty: Uint8Array, id: Uint8Array): Uint8Array {
  return allocationHelper([
    [TICKET_PREFIX.length, TICKET_PREFIX],
    [unAcknowledgedSubPrefix.length, unAcknowledgedSubPrefix],
    [COMPRESSED_PUBLIC_KEY_LENGTH, publicKeyCounterparty],
    [SEPERATOR.length, SEPERATOR],
    [id.length, id],
  ])
}

export async function UnAcknowledgedTicketsParse(arg: Uint8Array): Promise<[PeerId, Uint8Array]> {
  return [
    await pubKeyToPeerId(
      arg.slice(
        TICKET_PREFIX.length + unAcknowledgedSubPrefix.length,
        TICKET_PREFIX.length + unAcknowledgedSubPrefix.length + COMPRESSED_PUBLIC_KEY_LENGTH
      )
    ),
    arg.slice(
      TICKET_PREFIX.length + unAcknowledgedSubPrefix.length + COMPRESSED_PUBLIC_KEY_LENGTH + SEPERATOR.length,
      TICKET_PREFIX.length +
        unAcknowledgedSubPrefix.length +
        COMPRESSED_PUBLIC_KEY_LENGTH +
        SEPERATOR.length +
        KEY_LENGTH
    ),
  ]
}

export function PacketTag(tag: Uint8Array): Uint8Array {
  return allocationHelper([
    [PACKET_PREFIX.length, PACKET_PREFIX],
    [packetTagSubPrefix.length, packetTagSubPrefix],
    [SEPERATOR.length, SEPERATOR],
    [tag.length, tag],
  ])
}

export const KeyPair = encoder.encode('keyPair')

type Config = [number, Uint8Array]

function allocationHelper(arr: Config[]) {
  const totalLength = arr.reduce((acc, current) => {
    return acc + current[0]
  }, 0)

  let result = new Uint8Array(totalLength)

  let offset = 0
  for (let [size, data] of arr) {
    result.set(data, offset)
    offset += size
  }

  return result
}
