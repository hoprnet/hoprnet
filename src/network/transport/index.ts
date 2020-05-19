import net from 'net'
import type { Socket } from 'net'
import mafmt from 'mafmt'
const errCode = require('err-code')
const log = require('debug')('libp2p:tcp')
import { socketToConn } from './socket-to-conn'

import abortable from 'abortable-iterator'
import AbortController from 'abort-controller'

// @ts-ignore
import libp2p = require('libp2p')

import { createListener, Listener } from './listener'
import { multiaddrToNetConfig } from './utils'
import { AbortError } from 'abortable-iterator'
import { CODE_CIRCUIT, CODE_P2P, RELAY_CIRCUIT_TIMEOUT, USE_OWN_STUN_SERVERS } from './constants'

import Multiaddr from 'multiaddr'
import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

import pipe from 'it-pipe'
import pushable from 'it-pushable'

import Peer, { Instance as SimplePeerInstance } from 'simple-peer'

// @ts-ignore
import wrtc = require('wrtc')

import { pubKeyToPeerId } from '../../utils'
import { u8aToHex, u8aEquals, u8aAdd } from '@hoprnet/hopr-utils'

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

import chalk from 'chalk'

const RELAY_REGISTER = '/hopr/relay-register/0.0.1'
const RELAY_UNREGISTER = '/hopr/relay-unregister/0.0.1'
const DELIVERY_REGISTER = '/hopr/delivery-register/0.0.1'
const DELIVERY_UNREGISTER = '/hopr/delivery-unregister/0.0.1'
const WEBRTC = '/hopr/webrtc/0.0.1'

const RELAY_DELIVER = (from: Uint8Array) => `/hopr/deliver${u8aToHex(from)}/0.0.1`
const RELAY_FORWARD = (from: Uint8Array, to: Uint8Array) => {
  if (from.length !== to.length) {
    throw Error(`Could not generate RELAY_FORWARD protocol string because array lengths do not match`)
  }

  return `/hopr/forward${u8aToHex(u8aAdd(false, from, to))}/0.0.1`
}

const OK = new TextEncoder().encode('OK')
const FAIL = new TextEncoder().encode('FAIL')

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
  private relays?: PeerInfo[]
  private stunServers: { urls: string }[]

  private _encoder: TextEncoder
  private _decoder: TextDecoder

  private connHandler: ConnHandler

  constructor({
    upgrader,
    libp2p,
    bootstrapServers,
  }: {
    upgrader: Upgrader
    libp2p: libp2p
    bootstrapServers?: PeerInfo[]
  }) {
    if (!upgrader) {
      throw new Error('An upgrader must be provided. See https://github.com/libp2p/interface-transport#upgrader.')
    }

    if (!libp2p) {
      throw new Error('Transport module needs access to libp2p.')
    }

    if (bootstrapServers !== undefined && bootstrapServers.length > 0) {
      this.relays = bootstrapServers.filter(
        (peerInfo: PeerInfo) => peerInfo !== undefined && !libp2p.peerInfo.id.isEqual(peerInfo.id)
      )

      this.stunServers = []
      for (let i = 0; i < this.relays.length; i++) {
        let urls = ''
        this.relays[i].multiaddrs.forEach((ma: Multiaddr) => {
          if (urls.length > 0) {
            urls += ', '
          }

          const opts = ma.toOptions()

          if (opts.family == 'ipv4') {
            urls += `stun:${opts.host}`
          } else if (opts.family == 'ipv6') {
            // WebRTC seems to have no support IPv6 addresses
            // urls += `stun:[0${opts.host}]`
          }
        })
        this.stunServers.push({ urls })
      }
    }

    this._registrar = libp2p.registrar
    this._handle = libp2p.handle.bind(libp2p)
    this._unhandle = libp2p.unhandle.bind(libp2p)
    this._dialer = libp2p.dialer
    this._peerInfo = libp2p.peerInfo
    this._upgrader = upgrader

    this._encoder = new TextEncoder()
    this._decoder = new TextDecoder()

    this._handle(RELAY_REGISTER, this.handleRelayRegister.bind(this))
    this._handle(RELAY_UNREGISTER, this.handleRelayUnregister.bind(this))
    this._handle(DELIVERY_REGISTER, this.handleDeliveryRegister.bind(this))
    this._handle(DELIVERY_UNREGISTER, this.handleDeliveryUnregister.bind(this))
    this._handle(WEBRTC, this.handleWebRTC.bind(this))
  }

  private relayToConn(options: { stream: Stream; counterparty: PeerId; relay: PeerId }): MultiaddrConnection {
    const maConn: MultiaddrConnection = {
      ...options.stream,
      conn: options.stream,
      remoteAddr: Multiaddr(`/p2p/${options.counterparty.toB58String()}`),
      close: async (err?: Error) => {
        if (err !== undefined) {
          console.log(err)
        }

        await this.closeConnection(options.counterparty, options.relay)

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
          relay: connection.remotePeer,
        })
      )

      if (this.connHandler !== undefined) {
        return this.connHandler(conn)
      }
    }
  }

  forwardHandlerFactory(counterparty: PeerId): (handler: Handler) => void {
    return (async ({ stream, connection }: Handler) => {
      let conn = this._registrar.getConnection(new PeerInfo(counterparty))

      if (!conn) {
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
      }

      const { stream: innerStream } = await conn.newStream([RELAY_DELIVER(connection.remotePeer.pubKey.marshal())])

      pipe(stream, innerStream, stream)
    }).bind(this)
  }

  handleDeliveryUnregister({ stream }: Handler) {
    pipe(stream, async (source: AsyncIterable<Uint8Array>) => {
      for await (const msg of source) {
        let counterparty: PeerId
        try {
          counterparty = await pubKeyToPeerId(msg.slice())
        } catch {
          return
        }

        this._unhandle(RELAY_DELIVER(counterparty.pubKey.marshal()))
      }
    })
  }

  handleDeliveryRegister({ stream }: Handler) {
    pipe(
      stream,
      (source: AsyncIterable<Uint8Array>) => {
        return async function* (this: TCP) {
          for await (const msg of source) {
            let sender: PeerId
            try {
              sender = await pubKeyToPeerId(msg.slice())
            } catch {
              return yield FAIL
            }

            this._handle(RELAY_DELIVER(sender.pubKey.marshal()), this.deliveryHandlerFactory(sender))

            return yield OK
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
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          let counterparty: PeerId

          try {
            counterparty = await pubKeyToPeerId(msg.slice())
          } catch {
            return
          }

          this._unhandle(
            RELAY_FORWARD(
              /* prettier-ignore */
              connection.remotePeer.pubKey.marshal(),
              counterparty.pubKey.marshal()
            )
          )

          let conn = this._registrar.getConnection(new PeerInfo(counterparty))

          if (!conn) {
            try {
              conn = await this._dialer.connectToPeer(new PeerInfo(counterparty))
            } catch (err) {
              return
            }
          }

          const { stream: unRegisterStream } = await conn.newStream([DELIVERY_UNREGISTER])

          pipe(
            /* prettier-ignore */
            [counterparty.pubKey.marshal()],
            unRegisterStream
          )
        }
      }
    )
  }

  async closeConnection(counterparty: PeerId, relay: PeerId) {
    this._unhandle(RELAY_DELIVER(counterparty.pubKey.marshal()))

    // @TODO unregister at correct relay node
    let conn = this._registrar.getConnection(new PeerInfo(relay))

    if (!conn) {
      try {
        conn = await this._dialer.connectToPeer(new PeerInfo(relay))
      } catch (err) {
        console.log(
          `Could not request relayer ${relay.toB58String()} to tear down relayed connection. Error was:\n`,
          err
        )
        return
      }
    }

    const { stream: unRegisterStream } = await conn.newStream([RELAY_UNREGISTER])

    await pipe(
      /* prettier-ignore */
      [counterparty.pubKey.marshal()],
      unRegisterStream
    )

    return
  }

  async registerDelivery(outerconnection: Connection, counterparty: PeerId): Promise<Uint8Array> {
    let conn = this._registrar.getConnection(new PeerInfo(counterparty))

    const abort = new AbortController()

    const timeout = setTimeout(() => {
      abort.abort()
    }, RELAY_CIRCUIT_TIMEOUT)

    if (!conn) {
      try {
        conn = await this._dialer.connectToPeer(new PeerInfo(counterparty), { signal: abort.signal })
      } catch (err) {
        console.log(
          `[Relayer] Could not establish relayed connection to destination ${counterparty.toB58String()}. Err was:\n`,
          err
        )
        return
      }
    }

    const { stream: deliverRegisterStream } = await conn.newStream([DELIVERY_REGISTER])

    let answer = await pipe(
      /* prettier-ignore */
      [outerconnection.remotePeer.pubKey.marshal()],
      deliverRegisterStream,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          return msg.slice()
        }
      }
    )

    clearTimeout(timeout)

    return answer || FAIL
  }

  handleRelayRegister({ stream, connection }: Handler) {
    pipe(
      /* prettier-ignore */
      stream,
      (source: AsyncIterable<Uint8Array>) => {
        return async function* (this: TCP) {
          for await (const msg of source) {
            let counterparty: PeerId

            try {
              counterparty = await pubKeyToPeerId(msg.slice())
            } catch {
              return yield FAIL
            }

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
              return yield OK
            }

            if (!u8aEquals(answer, FAIL)) {
              console.log(`Received unexpected message from counterparty '${answer}'`)
            }

            return yield FAIL
          }
        }.apply(this)
      },
      stream
    )
  }

  handleWebRTC({ stream }: Handler) {
    const queue = pushable<Uint8Array>()

    let channel: SimplePeerInstance
    if (USE_OWN_STUN_SERVERS) {
      channel = new Peer({ wrtc, trickle: true, config: { iceServers: this.stunServers } })
    } else {
      channel = new Peer({ wrtc, trickle: true })
    }

    const done = (err?: Error, conn?: Connection) => {
      channel.removeListener('connect', onConnect)
      channel.removeListener('error', onError)
      channel.removeListener('signal', onSignal)

      if (err) {
        console.log(`WebRTC connection failed`)
      } else if (this.connHandler) {
        this.connHandler(conn)
      }
    }

    const onSignal = (msg: string) => {
      queue.push(this._encoder.encode(JSON.stringify(msg)))
    }

    const onConnect = async () => {
      done(undefined, await this._upgrader.upgradeInbound(socketToConn((channel as unknown) as Socket)))
    }

    const onError = (err?: Error) => {
      done(err)
    }

    channel.on('signal', onSignal)
    channel.once('connect', onConnect)
    channel.once('error', onConnect)

    pipe(queue, stream, async (source: AsyncIterable<Uint8Array>) => {
      for await (const msg of source) {
        channel.signal(this._decoder.decode(msg.slice()))
      }
    })
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
      const destination = PeerId.createFromCID(ma.getPeerId())

      if (this.relays === undefined) {
        throw Error(
          `Could not connect ${chalk.yellow(
            ma.toString()
          )} because there was no relay defined. Connection error was:\n${err}`
        )
      }

      // Check whether we know some relays that we can use
      const potentialRelays = this.relays?.filter((peerInfo: PeerInfo) => !peerInfo.id.isEqual(destination))

      if (potentialRelays == null || potentialRelays.length == 0) {
        throw Error(
          `Destination ${chalk.yellow(
            ma.toString()
          )} cannot be accessed and directly and there is no other relay node known. Connection error was:\n${err}`
        )
      }

      return await this.dialWithRelay(ma, potentialRelays, options)
    }
  }

  tryWebRTC(conn: Connection, counterparty: PeerId, options?: { signal: AbortSignal }): Promise<Connection> {
    return new Promise<Connection>(async (resolve, reject) => {
      const { stream } = await conn.newStream([WEBRTC])
      const queue = pushable<Uint8Array>()

      let channel: SimplePeerInstance
      if (USE_OWN_STUN_SERVERS) {
        channel = new Peer({
          wrtc,
          initiator: true,
          trickle: true,
          config: { iceServers: this.stunServers },
        })
      } else {
        channel = new Peer({
          wrtc,
          initiator: true,
          trickle: true,
        })
      }

      const done = (err?: Error, conn?: Connection) => {
        channel.removeListener('connect', onConnect)
        channel.removeListener('error', onError)
        channel.removeListener('signal', onSignal)
        options.signal && options.signal.removeEventListener('abort', onAbort)

        if (err) {
          reject(err)
        } else {
          resolve(conn)
        }
      }

      const onAbort = () => {
        channel.destroy()
        setImmediate(reject)
      }

      const onSignal = (data: string): void => {
        queue.push(this._encoder.encode(JSON.stringify(data)))
      }

      const onConnect = async (): Promise<void> => {
        done(undefined, await this._upgrader.upgradeOutbound(socketToConn((channel as unknown) as Socket)))
      }

      const onError = (err?: Error) => {
        done(err)
      }

      channel.on('signal', onSignal)

      channel.once('error', onError)

      channel.once('connect', onConnect)

      pipe(
        /* prettier-ignore */
        queue,
        stream,
        async (source: AsyncIterable<Uint8Array>) => {
          for await (const msg of source) {
            channel.signal(this._decoder.decode(msg.slice()))
          }
        }
      )
    })
  }

  async dialWithRelay(ma: Multiaddr, relays: PeerInfo[], options?: DialOptions): Promise<Connection> {
    const destination = PeerId.createFromCID(ma.getPeerId())

    let relayConnection = await Promise.race(
      relays.map(
        (relay: PeerInfo) =>
          new Promise<Connection>(async resolve => {
            log(
              `[${chalk.blue(this._peerInfo.id.toB58String())}] trying to call ${chalk.yellow(
                ma.toString()
              )} over relay node ${chalk.yellow(relay.id.toB58String())}`
            )

            let relayConnection = this._registrar.getConnection(relay)

            if (!relayConnection) {
              try {
                return resolve(await this._dialer.connectToPeer(relay, { signal: options?.signal }))
              } catch {}
            }
          })
      )
    )

    if (!relayConnection) {
      throw Error(
        `Unable to establish a connection to any known relay node. Tried ${chalk.yellow(
          relays.map((relay: PeerInfo) => relay.id.toB58String()).join(`, `)
        )}`
      )
    }

    const { stream: registerStream } = await relayConnection.newStream([RELAY_REGISTER])

    const answer = await pipe(
      /* prettier-ignore */
      [destination.pubKey.marshal()],
      registerStream,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          return msg.slice()
        }
      }
    )

    if (!u8aEquals(answer, OK)) {
      throw Error(`Register relaying failed. Received '${this._decoder.decode(answer)}'.`)
    }

    const { stream: msgStream } = await relayConnection.newStream([
      RELAY_FORWARD(this._peerInfo.id.pubKey.marshal(), destination.pubKey.marshal()),
    ])

    if (options.signal) {
      msgStream.source = abortable(msgStream.source, options.signal)
    }

    let conn = await this._upgrader.upgradeOutbound(
      this.relayToConn({
        stream: msgStream,
        counterparty: destination,
        relay: relayConnection.remotePeer,
      })
    )

    try {
      let webRTCConn = await this.tryWebRTC(conn, destination, { signal: options.signal })
      conn.close()

      return webRTCConn
    } catch (err) {
      console.log(err)
    }

    return conn
  }

  async dialDirectly(ma: Multiaddr, options?: DialOptions): Promise<Connection> {
    log(`[${chalk.blue(this._peerInfo.id.toB58String())}] dailing ${chalk.yellow(ma.toString())} directly`)
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

      log('dialing %j', cOpts)
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
