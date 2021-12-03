import type { Stream, StreamType } from '../types'
import type { Multiaddr } from 'multiaddr'
import type { AddressInfo } from 'net'

export { parseAddress } from './addrs'
export type { ValidAddress } from './addrs'
export { encodeWithLengthPrefix, decodeWithLengthPrefix } from './lengthPrefix'

function isAsyncStream<T>(iterator: AsyncIterable<T> | Iterable<T>): iterator is AsyncIterable<T> {
  if ((iterator as AsyncIterable<T>)[Symbol.asyncIterator]) {
    return true
  }
  return false
}
/**
 * Converts messages of a stream into Uint8Arrays.
 * @param source a stream
 * @returns a stream of Uint8Arrays
 */
type SourceType = StreamType | string
export type toU8aStream = ((source: AsyncIterator<SourceType>) => AsyncIterable<StreamType>) &
  ((source: Iterable<SourceType>) => Iterable<StreamType>)

export function toU8aStream(source: Stream<StreamType | string>['source']): Stream['source'] {
  if (isAsyncStream(source)) {
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
  } else {
    return (function* () {
      for (const msg of source) {
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
}

/**
 * Changes the behavior of the given iterator such that it
 * fetches new messages before they are consumed by the
 * consumer.
 * @param iterator an async iterator
 * @returns given iterator that eagerly fetches messages
 */
export type eagerIterator<T> = ((iterator: AsyncIterable<T>) => AsyncIterable<T>) &
  ((iterator: Iterable<T>) => Iterable<T>)
export function eagerIterator<T>(iterator: AsyncIterable<T> | Iterable<T>): AsyncIterable<T> | Iterable<T> {
  let _iterator: Iterator<T> | AsyncIterator<T>

  let received: IteratorResult<T>
  let result: IteratorResult<T> | Promise<IteratorResult<T>>

  if (isAsyncStream(iterator)) {
    _iterator = (iterator as AsyncIterable<T>)[Symbol.asyncIterator]()

    result = _iterator.next()
    return (async function* () {
      while (true) {
        received = await result
        if (received.done) {
          break
        }
        result = _iterator.next()
        yield received.value
      }
    })()
  } else {
    _iterator = (iterator as Iterable<T>)[Symbol.iterator]()

    let result = _iterator.next()
    return (function* () {
      while (true) {
        received = result

        if (received.done) {
          break
        }
        result = _iterator.next()
        yield received.value
      }
    })()
  }
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
