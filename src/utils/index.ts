import type { Stream, StreamType } from '../types'
import type { Multiaddr } from 'multiaddr'
import type { AddressInfo } from 'net'

export { encodeWithLengthPrefix, decodeWithLengthPrefix } from './lengthPrefix'

/**
 * Converts messages of a stream into Uint8Arrays.
 * @param source a stream
 * @returns a stream of Uint8Arrays
 */
export function toU8aStream(source: Stream<StreamType | string>['source']): Stream['source'] {
  return (async function* () {
    for await (const msg of source) {
      if (typeof msg === 'string') {
        yield new TextEncoder().encode(msg)
      } else if (Buffer.isBuffer(msg)) {
        yield msg
      } else {
        yield msg.slice()
      }
    }
  })()
}

/**
 * Changes the behavior of the given iterator such that it
 * fetches new messages before they are consumed by the
 * consumer.
 * @param iterator an async iterator
 * @returns given iterator that eagerly fetches messages
 */
export function eagerIterator<T>(iterator: AsyncIterator<T>): AsyncGenerator<T> {
  let result = iterator.next()
  let received: IteratorResult<T>

  return (async function* () {
    while (true) {
      received = await result

      if (received.done) {
        break
      }
      result = iterator.next()
      yield received.value
    }
  })()
}

export function nodeToMultiaddr(addr: AddressInfo): Parameters<typeof Multiaddr.fromNodeAddress>[0] {
  let family: 4 | 6
  switch (addr.family) {
    case 'IPv4':
      family = 4
      break
    case 'IPv6':
      family = 6
      break
    default:
      throw Error(`Invalid family. Got ${addr.family}`)
  }

  return {
    family,
    address: addr.address,
    port: addr.port
  }
}
