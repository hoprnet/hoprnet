import type { StreamType } from '../types.js'
import type { AddressInfo, Server as TCPServer } from 'net'
import type { Socket as UDPSocket } from 'dgram'
import type { Connection, MultiaddrConnection } from '@libp2p/interface-connection'
import type { PeerId } from '@libp2p/interface-peer-id'
import { peerIdFromBytes } from '@libp2p/peer-id'

import { isAnyAddress } from '@hoprnet/hopr-utils'

import { Multiaddr } from '@multiformats/multiaddr'
import { CODE_CIRCUIT, CODE_P2P } from '../constants.js'
import { type Uint8ArrayList, isUint8ArrayList } from 'uint8arraylist'
import { Components } from '@libp2p/interfaces/components'

export * from './addrs.js'
export * from './addressSorters.js'
export { encodeWithLengthPrefix, decodeWithLengthPrefix } from './lengthPrefix.js'

function isAsyncStream<T>(iterator: AsyncIterable<T> | Iterable<T>): iterator is AsyncIterable<T> {
  if ((iterator as AsyncIterable<T>)[Symbol.asyncIterator]) {
    return true
  }
  return false
}

type SourceType = StreamType | Uint8ArrayList | Buffer | string

/**
 * Converts messages of a stream into Uint8Arrays.
 * @param source a stream
 * @returns a stream of Uint8Arrays
 */
export function toU8aStream<K extends AsyncIterable<SourceType> | Iterable<SourceType>>(
  source: K
): K extends Iterable<any> ? Iterable<Uint8Array> : AsyncIterable<Uint8Array> {
  if (isAsyncStream<SourceType>(source)) {
    return (async function* () {
      for await (const msg of source) {
        if (typeof msg === 'string') {
          yield new TextEncoder().encode(msg)
        } else if (Buffer.isBuffer(msg)) {
          yield new Uint8Array(msg.buffer, msg.byteOffset, msg.byteLength)
        } else if (isUint8ArrayList(msg)) {
          yield msg.slice()
        } else {
          yield msg
        }
      }
    })() as any // Typescript limitation
  } else {
    return (function* () {
      for (const msg of source as Iterable<SourceType>) {
        if (typeof msg === 'string') {
          yield new TextEncoder().encode(msg)
        } else if (Buffer.isBuffer(msg)) {
          yield new Uint8Array(msg.buffer, msg.byteOffset, msg.byteLength)
        } else if (isUint8ArrayList(msg)) {
          yield msg.slice()
        } else {
          yield msg
        }
      }
    })() as any // Typescript limitation
  }
}

/**
 * Changes the behavior of the given iterator such that it
 * fetches new messages before they are consumed by the
 * consumer.
 * @param iterator an async iterator
 * @returns given iterator that eagerly fetches messages
 */
export function eagerIterator<T, K extends Iterable<T> | AsyncIterable<T>>(
  iterator: K
): K extends Iterable<any> ? Iterable<T> : AsyncIterable<T> {
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
    })() as any // Typescript limitation
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
    })() as any // Typescript limitation
  }
}

/**
 * Converts a Node.js address instance to a format that is
 * understood by Multiaddr
 * @param addr a Node.js address instance
 * @returns
 */
export function nodeToMultiaddr(addr: AddressInfo): Multiaddr {
  let address: string
  let family: 4 | 6
  switch (addr.family) {
    case 'IPv4':
      family = 4
      // Node.js tends answer `socket.address()` calls on `udp4`
      // sockets with `::` instead of `0.0.0.0`
      if (isAnyAddress(addr.address, 'IPv6')) {
        address = '0.0.0.0'
      } else {
        address = addr.address
      }
      break
    case 'IPv6':
      family = 6
      // Make sure that we use the right any address,
      // even if this is IPv4 any address
      if (isAnyAddress(addr.address, 'IPv4')) {
        address = '::'
      } else {
        address = addr.address
      }
      break
    default:
      throw Error(`Invalid family. Got ${addr.family}`)
  }

  let ma = Multiaddr.fromNodeAddress(
    {
      family,
      address,
      port: addr.port
    },
    'tcp'
  )

  return ma
}

/**
 * Binds a UDP or TCP socket to a port and a host
 * @param protocol type of the socket, either 'TCP' or 'UDP'
 * @param socket TCP Socket or UDP socket
 * @param logError forward error report, if any
 * @param opts host and port to bind to
 * @returns a Promise that resolves once the socket is bound
 */
export function bindToPort<T extends 'UDP' | 'TCP'>(
  protocol: T,
  socket: T extends 'TCP' ? TCPServer : UDPSocket,
  logError: (...args: any[]) => void,
  opts?: { host?: string; port: number }
): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    let done = false

    const errListener = (err: any) => {
      socket.removeListener('listening', successListener)
      if (!done) {
        done = true
        reject(err)
      }
    }

    const successListener = () => {
      socket.removeListener('error', errListener)
      if (!done) {
        done = true
        resolve()
      }
    }

    socket.once('error', errListener)
    socket.once('listening', successListener)

    try {
      switch (protocol) {
        case 'TCP':
          ;(socket as TCPServer).listen(opts)
          break
        case 'UDP':
          ;(socket as UDPSocket).bind(opts?.port)
          break
      }
    } catch (err: any) {
      socket.removeListener('error', errListener)
      socket.removeListener('listening', successListener)

      logError(`Could not bind to ${protocol} socket.`, err)

      if (!done) {
        done = true
        reject(err)
      }
    }
  })
}

/**
 * Attempts to close the given maConn. If a failure occurs, it will be logged.
 * @private
 * @param maConn
 */
export async function attemptClose(
  maConn: MultiaddrConnection | Connection | undefined,
  logError: (...args: any[]) => void
) {
  if (maConn == null) {
    return
  }

  try {
    await maConn.close()
  } catch (err) {
    logError?.('an error occurred while closing the connection', err)
  }
}

/**
 * Extracts the relay PeerId from a relay address
 * @param ma relay Address
 * @returns
 */
export function relayFromRelayAddress(ma: Multiaddr): PeerId {
  const tuples = ma.tuples() as [code: number, addr: Uint8Array][]

  if (tuples.length < 2 || tuples[0][0] != CODE_P2P || tuples[1][0] != CODE_CIRCUIT) {
    throw Error(`Cannot extract relay from non-relay address. Given address ${ma.toString()}`)
  }

  // Remove length prefix
  return peerIdFromBytes(tuples[0][1].slice(1))
}

export function cleanExistingConnections(
  components: Components,
  peer: PeerId,
  id: string,
  error: (...args: any[]) => void
) {
  for (const conn of components.getConnectionManager().getConnections(peer)) {
    if (conn.id === id) {
      continue
    }

    attemptClose(conn, error)
  }
}
