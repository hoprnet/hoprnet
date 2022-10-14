import net, { type Socket, type AddressInfo } from 'net'
import { abortableSource } from 'abortable-iterator'
import Debug from 'debug'
import { nodeToMultiaddr, toU8aStream } from '../utils/index.js'
// @ts-ignore untyped module
import retimer from 'retimer'

const log = Debug('hopr-connect:tcp')
const error = Debug('hopr-connect:tcp:error')
const verbose = Debug('hopr-connect:verbose:tcp')

// Timeout to wait for socket close before manually destroying it
const SOCKET_CLOSE_TIMEOUT = 2000

import type { MultiaddrConnection } from '@libp2p/interface-connection'

import type { Multiaddr } from '@multiformats/multiaddr'
import toIterable from 'stream-to-it'
import type { Stream, StreamSource, StreamSourceAsync } from '../types.js'

type SocketOptions = {
  signal?: AbortSignal
  closeTimeout?: number
}

/**
 * Class to encapsulate TCP sockets
 */
class TCPConnection implements MultiaddrConnection {
  public localAddr: Multiaddr

  public source: StreamSourceAsync
  public closed: boolean

  private _signal?: AbortSignal

  private closeTimeout: number

  public timeline: {
    open: number
    close?: number
  }

  constructor(public remoteAddr: Multiaddr, public socket: Socket, options?: SocketOptions) {
    this.localAddr = nodeToMultiaddr(this.socket.address() as AddressInfo)

    this.closed = false
    this._signal = options?.signal
    this.closeTimeout = options?.closeTimeout ?? SOCKET_CLOSE_TIMEOUT

    this.timeline = {
      open: Date.now()
    }

    this.socket.once('close', () => {
      // Whenever the socket gets closed, mark the
      // connection closed to cleanup data structures in
      // ConnectionManager
      this.timeline.close ??= Date.now()
    })

    this.source = this.createSource(this.socket)

    // Sink is passed as a function, so we need to explicitly bind it
    this.sink = this.sink.bind(this)
  }

  public close(): Promise<void> {
    if (this.socket.destroyed || this.closed) {
      return Promise.resolve()
    }
    this.closed = true

    return new Promise<void>((resolve, reject) => {
      let done = false

      const start = Date.now()

      const timer = retimer(() => {
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

        if (this.socket.destroyed) {
          log('%s:%s is already destroyed', cOptions.host, cOptions.port)
        } else {
          log(`destroying connection ${cOptions.host}:${cOptions.port}`)
          this.socket.destroy()
        }
      }, this.closeTimeout)

      // Resolve once closed
      // Could take place after timeout or as a result of `.end()` call
      this.socket.once('close', () => {
        if (done) {
          return
        }
        done = true
        timer.clear()

        resolve()
      })

      this.socket.once('error', (err: Error) => {
        log('socket error', err)

        // error closing socket
        this.timeline.close ??= Date.now()

        if (this.socket.destroyed) {
          done = true
          timer.clear()
        }

        reject(err)
      })

      // Send the FIN packet
      this.socket.end()

      if (this.socket.writableLength > 0) {
        // there are outgoing bytes waiting to be sent
        this.socket.once('drain', () => {
          log('socket drained')

          // all bytes have been sent we can destroy the socket (maybe) before the timeout
          this.socket.destroy()
        })
      } else {
        // nothing to send, destroy immediately
        this.socket.destroy()
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

  public async sink(source: StreamSource): Promise<void> {
    const u8aStream = toU8aStream(source)

    let iterableSink: Stream['sink']
    try {
      iterableSink = toIterable.sink<Uint8Array>(this.socket)

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
    this.socket.end()
  }

  /**
   * Tries to establish a TCP connection to the given address.
   *
   * @param ma Multiaddr to connect to
   * @param options
   * @returns Resolves to a TCP Socket, if successful
   */
  public static create(ma: Multiaddr, options?: SocketOptions): Promise<TCPConnection> {
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
