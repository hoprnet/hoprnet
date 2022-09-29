import net, { type Socket, type AddressInfo } from 'net'
import { abortableSource } from 'abortable-iterator'
import Debug from 'debug'
import { nodeToMultiaddr, toU8aStream } from '../utils/index.js'

const log = Debug('hopr-connect:tcp')
const error = Debug('hopr-connect:tcp:error')
const verbose = Debug('hopr-connect:verbose:tcp')

// Timeout to wait for socket close before destroying it
export const SOCKET_CLOSE_TIMEOUT = 1000

import type { MultiaddrConnection } from '@libp2p/interface-connection'

import type { Multiaddr } from '@multiformats/multiaddr'
import toIterable from 'stream-to-it'
import type { Stream, StreamSink, StreamSource, StreamSourceAsync } from '../types.js'
import type { DialOptions } from '@libp2p/interface-transport'

/**
 * Class to encapsulate TCP sockets
 */
class TCPConnection implements MultiaddrConnection {
  public localAddr: Multiaddr

  public sink: StreamSink
  public source: StreamSourceAsync
  public closed: boolean

  private _signal?: AbortSignal

  private keepAlive: boolean

  public timeline: {
    open: number
    close?: number
  }

  constructor(public remoteAddr: Multiaddr, public conn: Socket, options?: DialOptions) {
    this.localAddr = nodeToMultiaddr(this.conn.address() as AddressInfo)

    this.closed = false

    // @ts-ignore - hack
    this.keepAlive = options?.keepAlive ?? false

    this.timeline = new Proxy(
      {
        open: Date.now()
      },
      {
        set: (...args) => {
          console.log(args)
          if (args[1] === 'keepAlive' && args[2] == true) {
            this.keepAlive = true
            console.log(`keepAlive true`)
          }
          return Reflect.set(...args)
        }
      }
    )
    this.conn.once('close', () => {
      // Whenever the socket gets closed, mark the
      // connection closed to cleanup data structures in
      // ConnectionManager
      this.timeline.close ??= Date.now()
    })

    this._signal = options?.signal

    this.source = this.createSource(this.conn) as AsyncIterable<Uint8Array>
    this.sink = this._sink.bind(this)
  }

  public close(): Promise<void> {
    if (this.conn.destroyed || this.closed) {
      return Promise.resolve()
    }
    this.closed = true

    return new Promise<void>((resolve, reject) => {
      let done = false

      const start = Date.now()

      setTimeout(() => {
        if (done) {
          return
        }
        done = true

        const cOptions = this.remoteAddr.toOptions()
        log(
          'timeout closing socket to %s:%s after %dms, destroying it manually',
          cOptions.host,
          cOptions.port,
          Date.now() - start
        )

        if (this.conn.destroyed) {
          log('%s:%s is already destroyed', cOptions.host, cOptions.port)
        } else {
          log(`destroying connection ${cOptions.host}:${cOptions.port}`)
          this.conn.destroy()
        }
      }, SOCKET_CLOSE_TIMEOUT).unref()

      // Resolve once closed
      // Could take place after timeout or as a result of `.end()` call
      this.conn.once('close', () => {
        if (done) {
          return
        }
        done = true

        resolve()
      })

      this.conn.once('error', (err: Error) => {
        log('socket error', err)

        // error closing socket
        this.timeline.close ??= Date.now()

        if (this.conn.destroyed) {
          done = true
        }

        reject(err)
      })

      this.conn.end()

      if (this.conn.writableLength > 0) {
        // there are outgoing bytes waiting to be sent
        this.conn.once('drain', () => {
          log('socket drained')

          // all bytes have been sent we can destroy the socket (maybe) before the timeout
          this.conn.destroy()
        })
      } else {
        // nothing to send, destroy immediately
        this.conn.destroy()
      }
    })
  }

  private createSource(socket: net.Socket): AsyncIterable<Uint8Array> {
    const iterableSource = toU8aStream(toIterable.source<Uint8Array>(socket)) as AsyncIterable<Uint8Array>

    if (this._signal != undefined) {
      return abortableSource(iterableSource, this._signal)
    } else {
      return iterableSource
    }
  }

  private async _sink(source: StreamSource): Promise<void> {
    const u8aStream = toU8aStream(source)

    let iterableSink: Stream['sink']
    try {
      iterableSink = toIterable.sink<Uint8Array>(this.conn)

      try {
        await iterableSink(
          this._signal != undefined ? (abortableSource(u8aStream, this._signal) as StreamSource) : u8aStream
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
    } catch (err) {
      error(`TCP sink error`, err)
      return
    }

    // End the socket (= send FIN packet) unless otherwise requested
    if (!this.keepAlive) {
      this.conn.end()
    }
  }

  /**
   * @param ma Multiaddr to connect to
   * @param options
   * @returns Resolves a TCP Socket
   */
  public static create(ma: Multiaddr, options?: DialOptions): Promise<TCPConnection> {
    return new Promise<TCPConnection>((resolve, reject) => {
      const start = Date.now()
      const cOpts = ma.toOptions()

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

        resolve(new TCPConnection(ma, rawSocket, options))
      }

      rawSocket = net
        .createConnection({
          host: cOpts.host,
          port: cOpts.port,
          signal: options?.signal
        })
        .once('error', onError)
        .once('timeout', onTimeout)
        .once('connect', onConnect)
    })
  }

  public static fromSocket(socket: Socket) {
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

    // PeerId of remote peer is not yet known,
    // will be available after encryption is set up
    const remoteAddr = nodeToMultiaddr({
      address: socket.remoteAddress,
      port: socket.remotePort,
      family: socket.remoteFamily
    })

    return new TCPConnection(remoteAddr, socket)
  }
}

export { TCPConnection }
