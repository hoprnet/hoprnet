import net, { type Socket, type AddressInfo } from 'net'
import abortable from 'abortable-iterator'
import Debug from 'debug'
import { nodeToMultiaddr, toU8aStream } from '../utils'

const log = Debug('hopr-connect:tcp')
const error = Debug('hopr-connect:tcp:error')
const verbose = Debug('hopr-connect:verbose:tcp')

// Timeout to wait for socket close before destroying it
export const SOCKET_CLOSE_TIMEOUT = 1000

import type { MultiaddrConnection } from 'libp2p-interfaces/src/transport/types'

import type { Multiaddr } from 'multiaddr'
import toIterable from 'stream-to-it'
import type PeerId from 'peer-id'
import type { Stream, StreamSink, StreamSource, StreamSourceAsync, StreamType, HoprConnectDialOptions } from '../types'

/**
 * Class to encapsulate TCP sockets
 */
class TCPConnection implements MultiaddrConnection {
  public localAddr: Multiaddr
  public sink: StreamSink
  public source: StreamSourceAsync
  private closed: boolean | undefined

  private _stream: Stream

  private _signal?: AbortSignal

  public timeline: {
    open: number
    close?: number
  }

  constructor(public remoteAddr: Multiaddr, self: PeerId, public conn: Socket, options?: HoprConnectDialOptions) {
    this.localAddr = nodeToMultiaddr(this.conn.address() as AddressInfo, self)

    this.timeline = {
      open: Date.now()
    }

    this.conn.once('close', () => {
      if (!this.closed) {
        options?.onDisconnect?.(remoteAddr)
      }

      // Whenever the socket gets closed, mark the
      // connection closed to cleanup data structures in
      // ConnectionManager
      this.timeline.close ??= Date.now()
    })

    this._signal = options?.signal

    this._stream = toIterable.duplex<StreamType>(this.conn)

    this.sink = this._sink.bind(this)

    this.source =
      this._signal != undefined
        ? abortable(this._stream.source, this._signal)
        : (this._stream.source as AsyncIterable<StreamType>)
  }

  public close(): Promise<void> {
    if (this.conn.destroyed || this.closed) {
      return Promise.resolve()
    }
    this.closed = true

    return new Promise<void>((resolve) => {
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
      }, SOCKET_CLOSE_TIMEOUT)

      // Resolve once closed
      // Could take place after timeout or as a result of `.end()` call
      this.conn.once('close', () => {
        if (done) {
          return
        }
        done = true

        resolve()
      })

      try {
        this.conn.end()
      } catch (err) {
        this.conn.destroy()
      }
    })
  }

  private async _sink(source: StreamSource): Promise<void> {
    const u8aStream = toU8aStream(source)
    try {
      await this._stream.sink(
        this._signal != undefined ? (abortable(u8aStream, this._signal) as StreamSource) : u8aStream
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
  }

  /**
   * @param ma
   * @param self
   * @param options
   * @returns Resolves a TCP Socket
   */
  public static create(ma: Multiaddr, self: PeerId, options?: HoprConnectDialOptions): Promise<TCPConnection> {
    return new Promise<TCPConnection>((resolve, reject) => {
      const start = Date.now()
      const cOpts = ma.toOptions()

      let rawSocket: Socket
      let finished = false

      const onError = (err: any) => {
        if (err.code === 'ABORT_ERR') {
          verbose(`Abort to ${ma.toString()} after ${Date.now() - start} ms`, err)
        } else {
          verbose(`Error connecting to ${ma.toString()}.`, err)
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

        resolve(new TCPConnection(ma, self, rawSocket, options))
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

  public static fromSocket(socket: Socket, self: PeerId) {
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
    const remoteAddr = nodeToMultiaddr(
      {
        address: socket.remoteAddress,
        port: socket.remotePort,
        family: socket.remoteFamily
      },
      undefined
    )

    return new TCPConnection(remoteAddr, self, socket)
  }
}

export { TCPConnection }
