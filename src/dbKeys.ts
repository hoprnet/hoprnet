import { pubKeyToPeerId } from './utils'
import type PeerId from 'peer-id'

const encoder = new TextEncoder()
const PREFIX: Uint8Array = encoder.encode('tickets-')
const SEPERATOR: Uint8Array = encoder.encode('-')

const acknowledgedSubPrefix = encoder.encode('acknowledged-')
const unAcknowledgedSubPrefix = encoder.encode('unacknowledged-')

const COMPRESSED_PUBLIC_KEY_LENGTH = 33
const KEY_LENGTH = 32

export function AcknowledgedTickets(publicKeyCounterparty: Uint8Array, id: Uint8Array): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [acknowledgedSubPrefix.length, acknowledgedSubPrefix],
    [publicKeyCounterparty.length, publicKeyCounterparty],
    [SEPERATOR.length, SEPERATOR],
    [id.length, id]
  ])
}

export function UnAcknowledgedTickets(publicKeyCounterparty: Uint8Array, id: Uint8Array): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [unAcknowledgedSubPrefix.length, unAcknowledgedSubPrefix],
    [COMPRESSED_PUBLIC_KEY_LENGTH, publicKeyCounterparty],
    [SEPERATOR.length, SEPERATOR],
    [id.length, id]
  ])
}

export async function UnAcknowledgedTicketsParse(arg: Uint8Array): Promise<[PeerId, Uint8Array]> {
  return [
    await pubKeyToPeerId(
      arg.slice(PREFIX.length + unAcknowledgedSubPrefix.length, PREFIX.length + unAcknowledgedSubPrefix.length + COMPRESSED_PUBLIC_KEY_LENGTH)
    ),
    arg.slice(
      PREFIX.length + unAcknowledgedSubPrefix.length + COMPRESSED_PUBLIC_KEY_LENGTH + SEPERATOR.length,
      PREFIX.length + unAcknowledgedSubPrefix.length + COMPRESSED_PUBLIC_KEY_LENGTH + SEPERATOR.length + KEY_LENGTH
    )
  ]
}

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
