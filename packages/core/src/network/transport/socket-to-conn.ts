/// <reference path="../../@types/it-handshake.ts" />
/// <reference path="../../@types/stream-to-it.ts" />
/// <reference path="../../@types/libp2p-utils.ts" />

import abortable from 'abortable-iterator'
import debug from 'debug'
import toIterable from 'stream-to-it'

import toMultiaddr from 'libp2p-utils/src/ip-port-to-multiaddr'

import { MultiaddrConnection, Stream } from 'libp2p'
import type Multiaddr from 'multiaddr'
import type { Socket } from 'net'

const log = debug('libp2p:tcp:socket')
const error = debug('libp2p:tcp:socket:error')

const SOCKET_CLOSE_TIMEOUT = 2000

function toWebrtcMultiaddr(address: undefined | string, port: undefined | number) {
  if (!address || !port) {
    return undefined
  }

  try {
    return toMultiaddr(address, port)
  } catch (err) {
    error(err)
    // Account for mdns hostnames, just make it a local ip for now
    return toMultiaddr('0.0.0.0', port)
  }
}

// Convert a socket into a MultiaddrConnection
// https://github.com/libp2p/interface-transport#multiaddrconnection
export function socketToConn(
  socket: Socket,
  options?: {
    listeningAddr?: Multiaddr
    localAddr?: Multiaddr
    remoteAddr?: Multiaddr
    signal?: AbortSignal
  }
): MultiaddrConnection {
  options = options || {}

  // Check if we are connected on a unix path
  if (options.listeningAddr && options.listeningAddr.getPath()) {
    options.remoteAddr = options.listeningAddr
  }

  if (options.remoteAddr && options.remoteAddr.getPath()) {
    options.localAddr = options.remoteAddr
  }

  const { sink, source } = toIterable.duplex(socket)
  const maConn: MultiaddrConnection = {
    async sink(source) {
      if (options.signal) {
        source = abortable(source, options?.signal) as Stream['source']
      }

      try {
        await sink(
          (async function* () {
            for await (const chunk of source) {
              // Convert BufferList to Buffer
              yield Buffer.isBuffer(chunk) ? chunk : chunk.slice()
            }
          })()
        )
      } catch (err) {
        // If aborted we can safely ignore
        if (err.type !== 'aborted') {
          // If the source errored the socket will already have been destroyed by
          // toIterable.duplex(). If the socket errored it will already be
          // destroyed. There's nothing to do here except log the error & return.
          error(err)
        }
      }
    },

    // @ts-ignore
    source, //: options.signal ? abortable(source, options.signal) : source,

    conn: socket,

    localAddr: options.localAddr || toWebrtcMultiaddr(socket.localAddress, socket.localPort),

    // If the remote address was passed, use it - it may have the peer ID encapsulated
    remoteAddr: options.remoteAddr || toWebrtcMultiaddr(socket.remoteAddress, socket.remotePort),

    timeline: { open: Date.now() },

    close(): Promise<void> {
      if (socket.destroyed) return Promise.resolve()

      return new Promise<void>((resolve, reject) => {
        const start = Date.now()

        // Attempt to end the socket. If it takes longer to close than the
        // timeout, destroy it manually.
        const timeout = setTimeout(() => {
          const cOptions = maConn.remoteAddr?.toOptions()
          log(
            'timeout closing socket to %s:%s after %dms, destroying it manually',
            cOptions?.host,
            cOptions?.port,
            Date.now() - start
          )

          if (socket.destroyed) {
            log('%s:%s is already destroyed', cOptions?.host, cOptions?.port)
          } else {
            socket.destroy()
          }

          resolve()
        }, SOCKET_CLOSE_TIMEOUT)

        socket.once('close', () => clearTimeout(timeout))
        socket.end((err?: Error) => {
          maConn.timeline.close = Date.now()
          if (err) {
            error(err)
            return reject(err)
          }

          resolve()
        })
      })
    }
  }

  socket.once('close', () => {
    // In instances where `close` was not explicitly called,
    // such as an iterable stream ending, ensure we have set the close
    // timeline
    if (!maConn.timeline.close) {
      maConn.timeline.close = Date.now()
    }
  })

  return maConn
}
