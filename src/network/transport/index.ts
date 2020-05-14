import net from 'net'
import type { Socket } from 'net'
import mafmt from 'mafmt'
const errCode = require('err-code')
const log = require('debug')('libp2p:tcp')
import { socketToConn } from './socket-to-conn'

// @ts-ignore
import libp2p = require('libp2p')

import { createListener, Listener } from './listener'
import { multiaddrToNetConfig } from './utils'
import { AbortError } from 'abortable-iterator'
import { CODE_CIRCUIT, CODE_P2P } from './constants'

import Multiaddr from 'multiaddr'
import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

import pipe from 'it-pipe'

import { pubKeyToPeerId } from '../../utils'
import { u8aToHex, u8aEquals, u8aAdd } from '@hoprnet/hopr-utils'
const RELAY_REGISTER = '/hopr/relay-register/0.0.1'
const RELAY_UNREGISTER = '/hopr/relay-unregister/0.0.1'
const DELIVERY_REGISTER = '/hopr/delivery-register/0.0.1'
const RELAY_DELIVER = (from: Uint8Array) => `/hopr/deliver${u8aToHex(from)}/0.0.1`
const RELAY_FORWARD = (from: Uint8Array, to: Uint8Array) => {
  if (from.length !== to.length) {
    throw Error(`Could not generate RELAY_FORWARD protocol string because array lengths do not match`)
  }

  return `/hopr/forward${u8aToHex(u8aAdd(false, from, to))}/0.0.1`
}

const OK = new TextEncoder().encode('OK')
const FAIL = new TextEncoder().encode('FAIL')

import type {
  Connection,
  Upgrader,
  DialOptions,
  Registrar,
  Dialer,
  ConnHandler,
  Handler,
  Stream,
  MultiaddrConnection,
} from './types'

/**
 * @class TCP
 */
class TCP {
  get [Symbol.toStringTag]() {
    return 'TCP'
  }

  private _upgrader: Upgrader
  private _dialer: Dialer
  private _registrar: Registrar
  private _peerInfo: PeerInfo
  private _handle: (protocols: string[] | string, handler: (connection: Handler) => void) => void
  private _unhandle: (protocols: string[] | string) => void
  private relay?: PeerInfo

  private connHandler: ConnHandler

  constructor({ upgrader, libp2p, bootstrap }: { upgrader: Upgrader; libp2p: libp2p; bootstrap?: PeerInfo }) {
    if (!upgrader) {
      throw new Error('An upgrader must be provided. See https://github.com/libp2p/interface-transport#upgrader.')
    }

    if (!libp2p) {
      throw new Error('Transport module needs access to libp2p.')
    }

    this.relay = bootstrap
    this._registrar = libp2p.registrar
    this._handle = libp2p.handle.bind(libp2p)
    this._unhandle = libp2p.unhandle.bind(libp2p)
    this._dialer = libp2p.dialer
    this._peerInfo = libp2p.peerInfo
    this._upgrader = upgrader

    this._handle(RELAY_REGISTER, this.handleRelayRegister.bind(this))
    this._handle(RELAY_UNREGISTER, this.handleRelayUnregister.bind(this))
    this._handle(DELIVERY_REGISTER, this.handleDeliveryRegister.bind(this))
  }

  private relayToConn(options: { stream: Stream; counterparty: PeerId }): MultiaddrConnection {
    const maConn: MultiaddrConnection = {
      ...options.stream,
      conn: options.stream,
      remoteAddr: Multiaddr(`/p2p/${options.counterparty.toB58String()}`),
      close: async (err?: Error) => {
        if (err !== undefined) {
          console.log(err)
        }

        await this.closeConnection(options.counterparty)

        maConn.timeline.close = Date.now()
      },
      timeline: {
        open: Date.now(),
      },
    }

    return maConn
  }

  deliveryHandlerFactory(sender: PeerId): (handler: Handler) => void {
    return async ({ stream, connection }: Handler) => {
      const conn = await this._upgrader.upgradeInbound(
        this.relayToConn({
          stream,
          counterparty: sender,
        })
      )

      if (this.connHandler !== undefined) {
        return this.connHandler(conn)
      }
    }
  }

  forwardHandlerFactory(counterparty: PeerId): (handler: Handler) => void {
    return (async ({ stream, connection }: Handler) => {
      let conn: Connection

      try {
        conn = await this._dialer.connectToPeer(new PeerInfo(counterparty))
      } catch (err) {
        console.log(`Could not forward packet to ${counterparty.toB58String()}. Error was :\n`, err)
        try {
          pipe([FAIL], stream)
        } catch (err) {
          console.log(`Failed to inform counterparty ${connection.remotePeer.toB58String()}`)
        }

        return
      }

      const { stream: innerStream } = await conn.newStream([RELAY_DELIVER(connection.remotePeer.pubKey.marshal())])

      pipe(stream, innerStream, stream)
    }).bind(this)
  }

  handleDeliveryRegister({ stream }: Handler) {
    pipe(
      stream,
      (source: AsyncIterable<Uint8Array>) => {
        return async function* (this: TCP) {
          for await (const msg of source) {
            const sender = await pubKeyToPeerId(msg.slice())

            this._handle(RELAY_DELIVER(sender.pubKey.marshal()), this.deliveryHandlerFactory(sender))
            yield OK
          }
        }.apply(this)
      },
      stream
    )
  }

  handleRelayUnregister({ stream, connection }: Handler) {
    pipe(
      /* prettier-ignore */
      stream,
      (source: AsyncIterable<Uint8Array>) => {
        return async function* (this: TCP) {
          for await (const msg of source) {
            const counterparty = await pubKeyToPeerId(msg.slice())

            try {
              this._unhandle(
                RELAY_FORWARD(
                  /* prettier-ignore */
                  connection.remotePeer.pubKey.marshal(),
                  counterparty.pubKey.marshal()
                )
              )
            } catch (err) {
              console.log(err)
            }

            let conn: Connection
            try {
              conn = await this._dialer.connectToPeer(new PeerInfo(counterparty))
            } catch (err) {}

            yield OK
          }
        }.apply(this)
      },
      stream
    )
  }

  async closeConnection(counterparty: PeerId) {
    this._unhandle(RELAY_DELIVER(counterparty.pubKey.marshal()))

    let conn: Connection

    try {
      conn = await this._dialer.connectToPeer(this.relay)
    } catch (err) {
      console.log(
        `Could not request relayer ${this.relay.id.toB58String()} to tear down relayed connection. Error was:\n`,
        err
      )
      return
    }

    const { stream } = await conn.newStream([RELAY_UNREGISTER])

    await pipe(
      /* prettier-ignore */
      [counterparty.pubKey.marshal()],
      stream,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          return msg.slice()
        }
      }
    )

    return
  }

  async registerDelivery(outerconnection: Connection, counterparty: PeerId): Promise<Uint8Array> {
    let conn: Connection

    try {
      conn = await this._dialer.connectToPeer(new PeerInfo(counterparty))
    } catch (err) {
      console.log(err)
      return
    }

    const { stream } = await conn.newStream([DELIVERY_REGISTER])

    return await pipe(
      /* prettier-ignore */
      [outerconnection.remotePeer.pubKey.marshal()],
      stream,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          return msg.slice()
        }
      }
    )
  }

  handleRelayRegister({ stream, connection }: Handler) {
    pipe(
      /* prettier-ignore */
      stream,
      (source: AsyncIterable<Uint8Array>) => {
        return async function* (this: TCP) {
          for await (const msg of source) {
            const counterparty = await pubKeyToPeerId(msg.slice())

            // setImmediate
            // make this non-blocking
            const answer = await this.registerDelivery(connection, counterparty)

            if (u8aEquals(answer, OK)) {
              this._handle(
                RELAY_FORWARD(
                  /* prettier-ignore */
                  connection.remotePeer.pubKey.marshal(),
                  counterparty.pubKey.marshal()
                ),
                this.forwardHandlerFactory(counterparty)
              )
              yield OK
            } else if (u8aEquals(answer, FAIL)) {
              yield FAIL
            } else {
              console.log(`Received unexpected message from counterparty '${answer}'`)
            }
          }
        }.apply(this)
      },
      stream
    )
  }

  /**
   * @async
   * @param {Multiaddr} ma
   * @param {object} options
   * @param {AbortSignal} options.signal Used to abort dial requests
   * @returns {Connection} An upgraded Connection
   */
  async dial(ma: Multiaddr, options?: DialOptions): Promise<Connection> {
    options = options || {}

    try {
      return await this.dialDirectly(ma, options)
    } catch (err) {
      if (this.relay === undefined) {
        throw err
      }

      return await this.dialWithRelay(ma, options)
    }
  }

  async dialWithRelay(ma: Multiaddr, options?: DialOptions): Promise<Connection> {
    const destinationPeerId = PeerId.createFromCID(ma.getPeerId())

    console.log(`dailing ${ma.toString()} over relay node`)

    let relayConnection = this._registrar.getConnection(this.relay)

    if (!relayConnection) {
      relayConnection = await this._dialer.connectToPeer(this.relay)
    }

    const { stream: registerStream } = await relayConnection.newStream([RELAY_REGISTER])

    const answer = await pipe(
      /* prettier-ignore */
      [destinationPeerId.pubKey.marshal()],
      registerStream,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          return msg.slice()
        }
      }
    )

    if (!u8aEquals(answer, OK)) {
      throw Error(`Register relaying failed. Received '${answer}'.`)
    }

    const { stream: msgStream } = await relayConnection.newStream([
      RELAY_FORWARD(this._peerInfo.id.pubKey.marshal(), destinationPeerId.pubKey.marshal()),
    ])

    return await this._upgrader.upgradeOutbound(
      this.relayToConn({
        stream: msgStream,
        counterparty: destinationPeerId,
      })
    )
  }

  async dialDirectly(ma: Multiaddr, options?: DialOptions): Promise<Connection> {
    console.log(`dailing ${ma.toString()} directly`)
    const socket = await this._connect(ma, options)
    const maConn = socketToConn(socket, { remoteAddr: ma, signal: options.signal })

    log('new outbound connection %s', maConn.remoteAddr)
    const conn = await this._upgrader.upgradeOutbound(maConn)

    log('outbound connection %s upgraded', maConn.remoteAddr)
    return conn
  }
  /**
   * @private
   * @param {Multiaddr} ma
   * @param {object} options
   * @param {AbortSignal} options.signal Used to abort dial requests
   * @returns {Promise<Socket>} Resolves a TCP Socket
   */
  _connect(ma: Multiaddr, options: DialOptions): Promise<Socket> {
    if (options.signal && options.signal.aborted) {
      throw new AbortError()
    }

    return new Promise<Socket>((resolve, reject) => {
      const start = Date.now()
      const cOpts = multiaddrToNetConfig(ma) as any

      console.log('dialing %j', cOpts)
      const rawSocket = net.connect(cOpts)

      const onError = (err: Error) => {
        err.message = `connection error ${cOpts.host}:${cOpts.port}: ${err.message}`
        done(err)
      }

      const onTimeout = () => {
        log('connnection timeout %s:%s', cOpts.host, cOpts.port)
        const err = errCode(new Error(`connection timeout after ${Date.now() - start}ms`), 'ERR_CONNECT_TIMEOUT')
        // Note: this will result in onError() being called
        rawSocket.emit('error', err)
      }

      const onConnect = () => {
        log('connection opened %j', cOpts)
        done()
      }

      const onAbort = () => {
        log('connection aborted %j', cOpts)
        rawSocket.destroy()
        done(new AbortError())
      }

      const done = (err?: Error) => {
        rawSocket.removeListener('error', onError)
        rawSocket.removeListener('timeout', onTimeout)
        rawSocket.removeListener('connect', onConnect)
        options.signal && options.signal.removeEventListener('abort', onAbort)

        if (err) return reject(err)
        resolve(rawSocket)
      }

      rawSocket.on('error', onError)
      rawSocket.on('timeout', onTimeout)
      rawSocket.on('connect', onConnect)
      options.signal && options.signal.addEventListener('abort', onAbort)
    })
  }

  /**
   * Creates a TCP listener. The provided `handler` function will be called
   * anytime a new incoming Connection has been successfully upgraded via
   * `upgrader.upgradeInbound`.
   * @param {*} [options]
   * @param {function(Connection)} handler
   * @returns {Listener} A TCP listener
   */
  createListener(options: any, handler: (connection: any) => void): Listener {
    if (typeof options === 'function') {
      handler = options
      options = {}
    }
    options = options || {}

    this.connHandler = handler

    return createListener({ handler, upgrader: this._upgrader }, options)
  }

  /**
   * Takes a list of `Multiaddr`s and returns only valid TCP addresses
   * @param multiaddrs
   * @returns Valid TCP multiaddrs
   */
  filter(multiaddrs: Multiaddr[]): Multiaddr[] {
    multiaddrs = Array.isArray(multiaddrs) ? multiaddrs : [multiaddrs]

    return multiaddrs.filter(ma => {
      if (ma.protoCodes().includes(CODE_CIRCUIT)) {
        return false
      }

      return mafmt.TCP.matches(ma.decapsulateCode(CODE_P2P))
    })
  }
}

export default TCP
