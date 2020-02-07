const encoder = new TextEncoder()
const PREFIX: Uint8Array = encoder.encode('tickets-')
const SEPERATOR: Uint8Array = encoder.encode('-')

const acknowledgedSubPrefix = encoder.encode('acknowledged-')
const unAcknowledgedSubPrefix = encoder.encode('unacknowledged-')

class DbKeys {
  AcknowledgedTickets(publicKeyCounterparty: Uint8Array, id: Uint8Array) {
    return allocationHelper([
      [PREFIX.length, PREFIX],
      [acknowledgedSubPrefix.length, acknowledgedSubPrefix],
      [publicKeyCounterparty.length, publicKeyCounterparty],
      [SEPERATOR.length, SEPERATOR],
      [id.length, id]
    ])
  }

  UnAcknowledgedTickets(publicKeyCounterparty: Uint8Array, id: Uint8Array) {
    return allocationHelper([
      [PREFIX.length, PREFIX],
      [unAcknowledgedSubPrefix.length, unAcknowledgedSubPrefix],
      [publicKeyCounterparty.length, publicKeyCounterparty],
      [SEPERATOR.length, SEPERATOR],
      [id.length, id]
    ])
  }
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

export { DbKeys }
