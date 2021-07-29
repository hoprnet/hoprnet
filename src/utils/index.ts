import type { HandlerProps } from 'libp2p'
import type LibP2P from 'libp2p'
import type { Stream, StreamType } from '../types'
import Debug from 'debug'
import AbortController, { AbortSignal } from 'abort-controller'
import PeerId from 'peer-id'
import { Multiaddr } from 'multiaddr'
import { AddressInfo } from 'net'

const verbose = Debug('hopr-connect:dialer:verbose')
const error = Debug('hopr-connect:dialer:error')

export * from './network'
export { encodeWithLengthPrefix, decodeWithLengthPrefix } from './lengthPrefix'

const DEFAULT_DHT_QUERY_TIMEOUT = 2000 // ms

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

export async function dialHelper(
  libp2p: LibP2P,
  destination: PeerId,
  protocol: string,
  opts:
    | {
        timeout?: number
        signal: AbortSignal
      }
    | {
        timeout: number
        signal?: AbortSignal
      }
): Promise<Omit<HandlerProps, 'connection'> | undefined> {
  let signal: AbortSignal
  let timeout: NodeJS.Timeout | undefined
  if (opts.signal == undefined) {
    const abort = new AbortController()
    signal = abort.signal
    timeout = setTimeout(() => {
      abort.abort()
      verbose(`timeout while querying ${destination.toB58String()}`)
    }, opts.timeout)
  } else {
    signal = opts.signal
  }

  let err: any
  let struct: Omit<HandlerProps, 'connection'> | undefined
  try {
    struct = await libp2p.dialProtocol(destination, protocol, { signal })
  } catch (_err) {
    err = _err
  }

  if (struct != null) {
    if (timeout != undefined) {
      clearTimeout(timeout)
    }
    return struct
  }

  if (signal.aborted) {
    return
  }

  if ((err != null || struct == null) && libp2p._dht == undefined) {
    if (timeout != undefined) {
      clearTimeout(timeout)
    }
    error(`Could not dial ${destination.toB58String()} directly and libp2p was started without a DHT.`)
    return
  }

  let addresses = (libp2p.peerStore.get(destination)?.addresses ?? []).map((addr: any) => addr.multiaddr.toString())

  // Try to get some fresh addresses from the DHT
  let dhtResponse:
    | {
        id: PeerId
        multiaddrs: Multiaddr[]
      }
    | undefined

  try {
    // Let libp2p populate its internal peerStore with fresh addresses
    dhtResponse = await (libp2p._dht as any).findPeer(destination, { timeout: DEFAULT_DHT_QUERY_TIMEOUT })
  } catch (err) {
    error(
      `Querying the DHT as peer ${libp2p.peerId.toB58String()} for ${destination.toB58String()} failed. ${err.message}`
    )
  }

  const newAddresses = (dhtResponse?.multiaddrs ?? []).filter((addr) => addresses.includes(addr.toString()))

  if (signal.aborted) {
    return
  }

  // Only start a dial attempt if we have received new addresses
  if (newAddresses.length == 0) {
    if (timeout != undefined) {
      clearTimeout(timeout)
    }
    return
  }

  try {
    struct = await libp2p.dialProtocol(destination, protocol, { signal })
    verbose(`Dial after DHT request successful`, struct)
  } catch (err) {
    error(`Using new addresses after querying the DHT did not lead to a connection. Cannot connect. ${err.message}`)
    return
  }

  if (struct != null) {
    if (timeout != undefined) {
      clearTimeout(timeout)
    }
    return struct
  }

  // Let Typescript figure out the correct type
  return
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
