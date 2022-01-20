import net, { type Socket, type AddressInfo } from 'net'
import abortable, { AbortError } from 'abortable-iterator'
import Debug from 'debug'
import { nodeToMultiaddr } from '../utils'
import { once } from 'events'

const log = Debug('hopr-connect:tcp')
const error = Debug('hopr-connect:tcp:error')
const verbose = Debug('hopr-connect:verbose:tcp')

export const SOCKET_CLOSE_TIMEOUT = 1000

import type { MultiaddrConnection } from 'libp2p-interfaces/transport'

import type { Multiaddr } from 'multiaddr'
import toIterable from 'stream-to-it'
import { toU8aStream } from '../utils'
import type PeerId from 'peer-id'
import type { Stream, StreamType, HoprConnectDialOptions } from '../types'

/**
 * Class to encapsulate TCP sockets
 */
class TCPConnection implements MultiaddrConnection<StreamType> {
  public localAddr: Multiaddr
  public sink: Stream['sink']
  public source: Stream['source']

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
      // In instances where `close` was not explicitly called,
      // such as an iterable stream ending, ensure we have set the close
      // timeline
      this.timeline.close ??= Date.now()
    })

    this._signal = options?.signal

    this._stream = toIterable.duplex<StreamType>(this.conn)

    this.sink = this._sink.bind(this)

    this.source =
      this._signal != undefined
        ? (abortable(this._stream.source, this._signal) as Stream['source'])
        : this._stream.source
  }

  public async close(): Promise<void> {
    if (this.conn.destroyed) {
      return
    }

    const closePromise = once(this.conn, 'close')

    const start = Date.now()

    // Attempt to end the socket. If it takes longer to close than the
    // timeout, destroy it manually.
    const timeout = setTimeout(() => {
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
        log(`destroying connection`)
        this.conn.destroy()
      }
    }, SOCKET_CLOSE_TIMEOUT)

    try {
      this.conn.end()
    } catch (err) {
      error(`Error while trying to close TCP connection`, err)
    }

    await closePromise

    clearTimeout(timeout)
  }

  private async _sink(source: Stream['source']): Promise<void> {
    const u8aStream = toU8aStream(source)
    try {
      await this._stream.sink(
        this._signal != undefined ? (abortable(u8aStream, this._signal) as Stream['source']) : u8aStream
      )
    } catch (err: any) {
      // If aborted we can safely ignore
      if (err.type !== 'aborted') {
        // If the source errored the socket will already have been destroyed by
        // toIterable.duplex(). If the socket errored it will already be
        // destroyed. There's nothing to do here except log the error & return.
        error(err.message)
      }
    }
  }

  /**
   * @param ma
   * @param options
   * @param options.signal Used to abort dial requests
   * @returns Resolves a TCP Socket
   */
  public static create(ma: Multiaddr, self: PeerId, options?: HoprConnectDialOptions): Promise<TCPConnection> {
    if (options?.signal?.aborted) {
      throw new AbortError()
    }

    return new Promise<TCPConnection>((resolve, reject) => {
      const start = Date.now()
      const cOpts = ma.toOptions()

      log('dialing %j', cOpts)
      let rawSocket: Socket

      const onError = (err: Error) => {
        verbose('Error connecting:', err.message)
        // ENETUNREACH
        // ECONNREFUSED
        // @TODO check error(s)
        err.message = `connection error ${cOpts.host}:${cOpts.port}: ${err.message}`
        done(err)
      }

      const onTimeout = () => {
        log('connnection timeout %s:%s', cOpts.host, cOpts.port)
        done(new Error(`connection timeout after ${Date.now() - start}ms`))
      }

      const onConnect = () => {
        log('connection opened %j', cOpts)
        done()
      }

      const done = (err?: Error) => {
        rawSocket?.removeListener('timeout', onTimeout)
        rawSocket?.removeListener('connect', onConnect)

        if (err) {
          rawSocket?.destroy()
          return reject(err)
        }

        resolve(new TCPConnection(ma, self, rawSocket))
      }

      rawSocket = net
        .createConnection({
          host: cOpts.host,
          port: cOpts.port,
          signal: options?.signal
        })
        .on('error', onError)
        .on('timeout', onTimeout)
        .on('connect', onConnect)
    })
  }

  public static fromSocket(socket: Socket, self: PeerId) {
    if (socket.remoteAddress == undefined || socket.remoteFamily == undefined || socket.remotePort == undefined) {
      throw Error(`Could not determine remote address`)
    }

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
