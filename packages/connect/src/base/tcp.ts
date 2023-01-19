import net, { isIPv6, type Socket } from 'net'
import { abortableSource } from 'abortable-iterator'
import Debug from 'debug'
import { ip6Lookup, IPV4_EMBEDDED_ADDRESS, nodeToMultiaddr } from '../utils/index.js'
// @ts-ignore untyped module
import retimer from 'retimer'

import { create_counter } from '@hoprnet/hopr-utils'

const log = Debug('hopr-connect:tcp')
const error = Debug('hopr-connect:tcp:error')
const verbose = Debug('hopr-connect:verbose:tcp')

// Timeout to wait for socket close before manually destroying it
const SOCKET_CLOSE_TIMEOUT = 2000

import type { MultiaddrConnection } from '@libp2p/interface-connection'

import type { Multiaddr } from '@multiformats/multiaddr'
import toIterable from 'stream-to-it'
import type { StreamSource } from '../types.js'

type SocketOptions = {
  signal?: AbortSignal
  closeTimeout?: number
}

const directPackets = create_counter('connect_counter_direct_packets', 'Number of directly sent packets (TCP)')

/**
 * Class to encapsulate TCP sockets
 */
export function TCPConnection(
  remoteAddr: Multiaddr,
  socket: Socket,
  onClose: () => void,
  options?: SocketOptions
): MultiaddrConnection {
  const closeTimeout = options?.closeTimeout ?? SOCKET_CLOSE_TIMEOUT

  let closed = false

  const timeline = {
    open: Date.now()
  } as MultiaddrConnection['timeline']

  socket.once('close', () => {
    // Whenever the socket gets closed, mark the
    // connection closed to cleanup data structures in
    // ConnectionManager
    timeline.close ??= Date.now()
    onClose()
  })

  // Assign various connection properties once we're sure that public keys match,
  // i.e. dialed node == desired destination

  // Set the SO_KEEPALIVE flag on socket to tell kernel to be more aggressive on keeping the connection up
  socket.setKeepAlive(true, 1000)

  socket.on('end', function () {
    log(`SOCKET END on connection to ${remoteAddr.toString()}: other end of the socket sent a FIN packet`)
  })

  socket.on('timeout', function () {
    log(`SOCKET TIMEOUT on connection to ${remoteAddr.toString()}`)
  })

  socket.on('error', function (e) {
    error(`SOCKET ERROR on connection to ${remoteAddr.toString()}: ' ${JSON.stringify(e)}`)
  })

  socket.on('close', (had_error) => {
    log(`SOCKET CLOSE on connection to ${remoteAddr.toString()}: error flag is ${had_error}`)
  })

  const close = (): Promise<void> => {
    if (socket.destroyed || closed) {
      return Promise.resolve()
    }
    closed = true

    return new Promise<void>((resolve, reject) => {
      let done = false

      const start = Date.now()

      const timer = retimer(() => {
        if (done) {
          return
        }
        done = true

        const cOptions = remoteAddr.toOptions()
        log(
          'timeout closing socket to %s:%s after %dms, destroying it manually',
          cOptions.host,
          cOptions.port,
          Date.now() - start
        )

        if (socket.destroyed) {
          log('%s:%s is already destroyed', cOptions.host, cOptions.port)
        } else {
          log(`destroying connection ${cOptions.host}:${cOptions.port}`)
          socket.destroy()
        }
      }, closeTimeout)

      // Resolve once closed
      // Could take place after timeout or as a result of `.end()` call
      socket.once('close', () => {
        if (done) {
          return
        }
        done = true
        timer.clear()

        resolve()
      })

      socket.once('error', (err: Error) => {
        log('socket error', err)

        // error closing socket
        timeline.close ??= Date.now()
        onClose()

        if (socket.destroyed) {
          done = true
          timer.clear()
        }

        reject(err)
      })

      // Send the FIN packet
      socket.end()

      if (socket.writableLength > 0) {
        // there are outgoing bytes waiting to be sent
        socket.once('drain', () => {
          log('socket drained')

          // all bytes have been sent we can destroy the socket (maybe) before the timeout
          socket.destroy()
        })
      } else {
        // nothing to send, destroy immediately
        socket.destroy()
      }
    })
  }

  const sinkEndpoint = async (source: StreamSource): Promise<void> => {
    try {
      await sink(
        (async function* (): AsyncIterable<Uint8Array> {
          for await (const msg of options?.signal != undefined
            ? (abortableSource(source, options.signal) as StreamSource)
            : source) {
            yield msg
            directPackets.increment()
          }
        })()
      )
    } catch (err: any) {
      // If aborted we can safely ignore
      if (err.code !== 'ABORT_ERR' && err.type !== 'aborted') {
        // If the source errored the socket will already have been destroyed by
        // toIterable.duplex(). If the socket errored it will already be
        // destroyed. There's nothing to do here except log the error & return.
        error(`unexpected error in TCP sink function`, err)
      }
    }

    // End the socket (= send FIN packet) unless otherwise requested
    socket.end()
  }
  const { sink, source } = toIterable.duplex<Uint8Array>(socket)

  const iterableSource = (async function* () {
    // Node.js emits Buffer instances, so turn them into
    // proper Uint8Arrays
    for await (const chunk of source as AsyncIterable<Uint8Array>) {
      yield new Uint8Array(chunk.buffer, chunk.byteOffset, chunk.byteLength)
    }
  })()

  return {
    remoteAddr,
    timeline,
    close,
    source: options?.signal != undefined ? abortableSource(iterableSource, options.signal) : iterableSource,
    sink: sinkEndpoint
  }
}

export async function createTCPConnection(
  ma: Multiaddr,
  onClose: () => void,
  options?: SocketOptions
): Promise<MultiaddrConnection> {
  return new Promise<MultiaddrConnection>((resolve, reject) => {
    const start = Date.now()
    const cOpts = ma.toOptions()

    if (!isIPv6(cOpts.host)) {
      cOpts.host = `::ffff:${cOpts.host}`
    }

    let rawSocket: Socket
    let finished = false

    const onError = (err: any) => {
      if (err.code === 'ABORT_ERR') {
        verbose(`Abort to ${ma.toString()} after ${Date.now() - start} ms`)
      } else {
        verbose(`Error connecting to ${ma.toString()}.`)
      }

      done(err)
    }

    const onTimeout = () => {
      verbose(`Connection timeout while connecting to ${ma.toString()}`)
      done(new Error(`connection timeout after ${Date.now() - start}ms`))
    }

    const onConnect = () => {
      verbose(`Connection successful to ${ma.toString()}`)
      done()
    }

    const done = (err?: Error) => {
      if (finished) {
        return
      }
      finished = true

      // Make sure that `done` is called only once
      rawSocket?.removeListener('error', onError)
      rawSocket?.removeListener('timeout', onTimeout)
      rawSocket?.removeListener('connect', onConnect)

      if (err) {
        rawSocket?.destroy()
        return reject(err)
      }

      resolve(TCPConnection(ma, rawSocket, onClose, options) as any)
    }

    rawSocket = net
      .createConnection({
        host: cOpts.host,
        port: cOpts.port,
        signal: options?.signal,
        family: 6,
        lookup: ip6Lookup
      })
      .once('error', onError)
      .once('timeout', onTimeout)
      .once('connect', onConnect)
  })
}

export function fromSocket(socket: Socket, onClose: () => void) {
  if (socket.remoteAddress == undefined || socket.remoteFamily == undefined || socket.remotePort == undefined) {
    throw Error(`Could not determine remote address`)
  }

  // Catch error from *incoming* socket
  socket.once('error', (err: any) => {
    error(`Error in incoming socket ${socket.remoteAddress}`, err)
    try {
      socket.destroy()
    } catch (internalError: any) {
      error(`Error while destroying incoming socket that threw an error`, internalError)
    }
  })

  let remoteAddress: string
  let remoteFamily: 'IPv4' | 'IPv6'
  const ipv4Embedded = socket.remoteAddress.match(IPV4_EMBEDDED_ADDRESS)
  if (ipv4Embedded) {
    remoteAddress = ipv4Embedded[0]
    remoteFamily = 'IPv4'
  } else {
    remoteAddress = socket.remoteAddress
    remoteFamily = socket.remoteFamily as 'IPv4' | 'IPv6'
  }
  // PeerId of remote peer is not yet known,
  // will be available after encryption is set up
  const remoteAddr = nodeToMultiaddr({
    address: remoteAddress,
    port: socket.remotePort,
    family: remoteFamily
  })

  return TCPConnection(remoteAddr, socket, onClose)
}
