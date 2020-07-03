import net from 'net'
import abortable, { AbortError } from 'abortable-iterator'

import type { Socket } from 'net'
import mafmt from 'mafmt'
import errCode from 'err-code'

import debug from 'debug'
const log = debug('hopr-core:transport')
const error = debug('hopr-core:transport:error')

import { socketToConn } from './socket-to-conn'

import AbortController from 'abort-controller'

// @ts-ignore
import handshake = require('it-handshake')

import myHandshake from './handshake'

// @ts-ignore
import libp2p = require('libp2p')

import { createListener, Listener } from './listener'
import { multiaddrToNetConfig } from './utils'
import { USE_WEBRTC, CODE_P2P, RELAY_CIRCUIT_TIMEOUT, USE_OWN_STUN_SERVERS } from './constants'

import Multiaddr from 'multiaddr'
import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

import pipe from 'it-pipe'

import { pubKeyToPeerId } from '../../utils'
import { u8aEquals } from '@hoprnet/hopr-utils'

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

import pushable, { Pushable } from 'it-pushable'

import upgradeToWebRTC from './webrtc'

const RELAY_REGISTER = '/hopr/relay-register/0.0.1'
const DELIVERY_REGISTER = '/hopr/delivery-register/0.0.1'

const OK = new TextEncoder().encode('OK')
const FAIL = new TextEncoder().encode('FAIL')

/**
 * @class TCP
 */
class TCP {
  get [Symbol.toStringTag]() {
    // @TODO change this to sth more meaningful
    return 'TCP'
  }

  private _useWebRTC: boolean
  private _useOwnStunServers: boolean
  // ONLY FOR TESTING
  private _failIntentionallyOnWebRTC: boolean
  private _timeoutIntentionallyOnWebRTC?: Promise<void>
  // END ONLY FOR TESTING
  private _upgrader: Upgrader
  private _dialer: Dialer
  private _registrar: Registrar
  private _peerInfo: PeerInfo
  private _handle: (protocols: string[] | string, handler: (connection: Handler) => void) => void
  private relays?: PeerInfo[]
  private stunServers: { urls: string }[]

  private connHandler: ConnHandler

  constructor({
    upgrader,
    libp2p,
    bootstrapServers,
    useWebRTC,
    useOwnStunServers,
    failIntentionallyOnWebRTC,
    timeoutIntentionallyOnWebRTC,
  }: {
    upgrader: Upgrader
    libp2p: libp2p
    bootstrapServers?: PeerInfo[]
    useWebRTC?: boolean
    useOwnStunServers?: boolean
    failIntentionallyOnWebRTC?: boolean
    timeoutIntentionallyOnWebRTC?: Promise<void>
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

    this._timeoutIntentionallyOnWebRTC = timeoutIntentionallyOnWebRTC
    this._failIntentionallyOnWebRTC = failIntentionallyOnWebRTC || false
    this._useOwnStunServers = useOwnStunServers === undefined ? USE_OWN_STUN_SERVERS : useOwnStunServers
    this._useWebRTC = useWebRTC === undefined ? USE_WEBRTC : useWebRTC
    this._registrar = libp2p.registrar
    this._handle = libp2p.handle.bind(libp2p)
    this._dialer = libp2p.dialer
    this._peerInfo = libp2p.peerInfo
    this._upgrader = upgrader

    this._handle(RELAY_REGISTER, this.handleRelay.bind(this))
    this._handle(DELIVERY_REGISTER, this.handleDelivery.bind(this))
  }

  async handleRelayConnection(stream: Stream): Promise<{ stream: Stream; counterparty: PeerId }> {
    let shaker = handshake(stream)

    let sender: PeerId

    const pubKeySender = (await shaker.read())?.slice()

    if (pubKeySender == null) {
      error(`Received empty message. Ignoring connection ...`)
      shaker.write(FAIL)
      shaker.rest()
      return
    }

    try {
      sender = await pubKeyToPeerId(pubKeySender)
    } catch (err) {
      error(`Could not decode sender peerId. Error was: ${err}`)
      shaker.write(FAIL)
      shaker.rest()
      return
    }

    shaker.write(OK)
    shaker.rest()

    return { stream: shaker.stream, counterparty: sender }
  }

  async handleDelivery({ stream, connection }: Handler) {
    const relayStream = await this.handleRelayConnection(stream)

    let conn: Connection

    let webRTCsendBuffer: Pushable<Uint8Array>
    let webRTCrecvBuffer: Pushable<Uint8Array>

    if (this._useWebRTC) {
      webRTCsendBuffer = pushable<Uint8Array>()
      webRTCrecvBuffer = pushable<Uint8Array>()
    }

    const myStream = myHandshake(webRTCsendBuffer, webRTCrecvBuffer)

    pipe(
      // prettier-ignore
      relayStream.stream.source,
      myStream.webRtcStream.source
    )

    pipe(
      // prettier-ignore
      myStream.webRtcStream.sink,
      relayStream.stream.sink
    )

    if (this._useWebRTC) {
      try {
        let socket = await upgradeToWebRTC(webRTCsendBuffer, webRTCrecvBuffer, {
          _timeoutIntentionallyOnWebRTC: this._timeoutIntentionallyOnWebRTC,
          _failIntentionallyOnWebRTC: this._failIntentionallyOnWebRTC,
        })

        webRTCrecvBuffer.end()
        webRTCsendBuffer.end()

        conn = await this._upgrader.upgradeInbound(
          socketToConn(socket, {
            remoteAddr: Multiaddr(`/p2p/${relayStream.counterparty.toB58String()}`),
            localAddr: Multiaddr(`/p2p/${this._peerInfo.id.toB58String()}`),
          })
        )
      } catch (err) {
        error(err)

        webRTCrecvBuffer.end()
        webRTCsendBuffer.end()

        conn = await this._upgrader.upgradeInbound(
          this.relayToConn({
            stream: myStream.relayStream,
            counterparty: relayStream.counterparty,
            connection,
          })
        )
      }
    } else {
      try {
        conn = await this._upgrader.upgradeInbound(
          this.relayToConn({
            stream: myStream.relayStream,
            counterparty: relayStream.counterparty,
            connection,
          })
        )
      } catch (err) {
        error(err)
        return
      }
    }

    this.connHandler?.call(conn)
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

    let error: Error
    if (['ip4', 'ip6', 'dns4', 'dns6'].includes(ma.protoNames()[0])) {
      try {
        return await this.dialDirectly(ma, options)
      } catch (err) {
        if (err.type === 'aborted') {
          throw err
        }
        error = err
      }
    }

    if (this.relays === undefined) {
      throw Error(
        `Could not connect ${chalk.yellow(ma.toString())} because there was no relay defined.${
          error != null ? ` Connection error was:\n${error}` : ''
        }`
      )
    }

    const destination = PeerId.createFromCID(ma.getPeerId())

    // Check whether we know some relays that we can use
    const potentialRelays = this.relays?.filter((peerInfo: PeerInfo) => !peerInfo.id.isEqual(destination))

    if (potentialRelays == null || potentialRelays.length == 0) {
      throw Error(
        `Destination ${chalk.yellow(
          ma.toString()
        )} cannot be accessed and directly and there is no other relay node known.${
          error != null ? ` Connection error was:\n${error}` : ''
        }`
      )
    }

    return await this.dialWithRelay(ma, potentialRelays, options)
  }

  async establishRelayedConnection(ma: Multiaddr, relays: PeerInfo[], options?: DialOptions) {
    const destination = PeerId.createFromCID(ma.getPeerId())

    if (options.signal?.aborted) {
      throw new AbortError()
    }

    let [relayConnection, index] = await Promise.race(
      relays.map(
        async (relay: PeerInfo, index: number): Promise<[Connection, number]> => {
          let relayConnection = this._registrar.getConnection(relay)

          if (!relayConnection) {
            relayConnection = await this._dialer.connectToPeer(relay, { signal: options?.signal })
          }

          return [relayConnection, index]
        }
      )
    )

    if (!relayConnection) {
      throw Error(
        `Unable to establish a connection to any known relay node. Tried ${chalk.yellow(
          relays.map((relay: PeerInfo) => relay.id.toB58String()).join(`, `)
        )}`
      )
    }

    if (options.signal?.aborted) {
      try {
        await relayConnection.close()
      } catch (err) {
        error(err)
      }
      throw new AbortError()
    }

    let { stream } = await relayConnection.newStream([RELAY_REGISTER])

    const shaker = handshake(stream)

    shaker.write(destination.pubKey.marshal())

    let answer: Buffer | undefined
    try {
      answer = (await shaker.read())?.slice()
    } catch (err) {
      error(err)
    }

    shaker.rest()

    if (answer == null || !u8aEquals(answer, OK)) {
      throw Error(
        `Could not establish relayed connection to ${chalk.blue(destination.toB58String())} over relay ${relays[
          index
        ].id.toB58String()}. Answer was: <${new TextDecoder().decode(answer)}>`
      )
    }

    return {
      stream: shaker.stream,
      baseConn: relayConnection,
    }
  }

  async dialWithRelay(ma: Multiaddr, relays: PeerInfo[], options?: DialOptions): Promise<Connection> {
    let webRTCsendBuffer: Pushable<Uint8Array>
    let webRTCrecvBuffer: Pushable<Uint8Array>

    const destination = PeerId.createFromCID(ma.getPeerId())

    const relayConnection = await this.establishRelayedConnection(ma, relays, options)

    let conn: Connection

    // if (options?.signal?.aborted) {
    //   try {
    //     await relayConnection.baseConn.close()
    //   } catch (err) {
    //     error(err)
    //   }

    //   if (this._useWebRTC) {
    //     webRTCsendBuffer?.end()
    //     webRTCrecvBuffer?.end()
    //   }

    //   throw new AbortError()
    // }

    const stream = myHandshake(webRTCsendBuffer, webRTCrecvBuffer, { signal: options.signal })

    pipe(
      // prettier-ignore
      relayConnection.stream.source,
      stream.webRtcStream.source
    )

    pipe(
      // prettier-ignore
      stream.webRtcStream.sink,
      relayConnection.stream.sink
    )

    if (this._useWebRTC) {
      try {
        webRTCsendBuffer = pushable<Uint8Array>()
        webRTCrecvBuffer = pushable<Uint8Array>()

        let socket = await upgradeToWebRTC(webRTCsendBuffer, webRTCrecvBuffer, {
          initiator: true,
          _timeoutIntentionallyOnWebRTC: this._timeoutIntentionallyOnWebRTC,
          _failIntentionallyOnWebRTC: this._failIntentionallyOnWebRTC,
        })

        webRTCsendBuffer.end()
        webRTCrecvBuffer.end()

        conn = await this._upgrader.upgradeOutbound(
          socketToConn(socket, {
            signal: options.signal,
            remoteAddr: Multiaddr(`/p2p/${destination.toB58String()}`),
            localAddr: Multiaddr(`/p2p/${this._peerInfo.id.toB58String()}`),
          })
        )
      } catch (err) {
        webRTCsendBuffer.end()
        webRTCrecvBuffer.end()

        conn = await this._upgrader.upgradeOutbound(
          this.relayToConn({
            stream: stream.relayStream,
            counterparty: destination,
            connection: relayConnection.baseConn,
          })
        )
      }
    } else {
      try {
        conn = await this._upgrader.upgradeOutbound(
          this.relayToConn({
            stream: stream.relayStream,
            counterparty: destination,
            connection: relayConnection.baseConn,
          })
        )
      } catch (err) {
        error(err)
        throw err
      }
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
        options.signal?.removeEventListener('abort', onAbort)

        if (err) {
          return reject(err)
        }
        resolve(rawSocket)
      }

      rawSocket.on('error', onError)
      rawSocket.on('timeout', onTimeout)
      rawSocket.on('connect', onConnect)
      options.signal?.addEventListener('abort', onAbort)
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

    return multiaddrs.filter((ma: Multiaddr) => {
      return mafmt.TCP.matches(ma.decapsulateCode(CODE_P2P)) || mafmt.P2P.matches(ma)
    })
  }

  async handleRelay({ stream, connection }: Handler) {
    const shaker = handshake(stream)

    let counterparty: PeerId
    let pubKeySender: Buffer | undefined

    try {
      pubKeySender = (await shaker.read())?.slice()
    } catch (err) {
      error(err)
    }

    if (pubKeySender == null) {
      error(
        `Received empty message from peer ${chalk.yellow(connection.remotePeer.toB58String())} during connection setup`
      )
      shaker.write(FAIL)
      shaker.rest()
      return
    }

    try {
      counterparty = await pubKeyToPeerId(pubKeySender)
    } catch (err) {
      log(
        `Peer ${chalk.yellow(
          connection.remotePeer.toB58String()
        )} asked to establish relayed connection to invalid counterparty. Error was ${err}. Received message ${pubKeySender}`
      )
      shaker.write(FAIL)
      shaker.rest()
      return
    }

    const abort = new AbortController()

    const timeout = setTimeout(() => abort.abort(), RELAY_CIRCUIT_TIMEOUT)

    let conn: Connection
    try {
      conn = this._registrar.getConnection(new PeerInfo(counterparty))

      if (!conn) {
        conn = await this._dialer.connectToPeer(new PeerInfo(counterparty), { signal: abort.signal })
      }
    } catch (err) {
      error(`Could not establish connection to ${counterparty.toB58String()}. Error was: ${err.message}`)
      clearTimeout(timeout)
      shaker.write(FAIL)
      shaker.rest()
      return
    }

    const { stream: deliveryStream } = await conn.newStream([DELIVERY_REGISTER])

    clearTimeout(timeout)

    const relayShaker = handshake(deliveryStream)

    relayShaker.write(connection.remotePeer.pubKey.marshal())

    let answer: Buffer | undefined
    try {
      answer = (await relayShaker.read())?.slice()
    } catch (err) {
      error(err)
    }

    if (answer == null || !u8aEquals(answer, OK)) {
      error(`Could not relay to peer ${counterparty.toB58String()} because we are unable to deliver packets.`)

      shaker.write(FAIL)

      shaker.rest()
      relayShaker.rest()

      return
    }

    shaker.write(OK)

    shaker.rest()
    relayShaker.rest()

    pipe(
      // prettier-ignore
      shaker.stream,
      // (source: AsyncIterable<Uint8Array>) => {
      //   return (async function* () {
      //     for await (const msg of source) {
      //       console.log(new TextDecoder().decode(msg))
      //     }
      //   })()
      // },
      relayShaker.stream
    )

    pipe(
      // prettier-ignore
      relayShaker.stream,
      shaker.stream
    )

    relayShaker.stream
  }

  private relayToConn({
    connection,
    stream,
    counterparty,
  }: {
    connection: Connection
    stream: Stream
    counterparty: PeerId
  }): MultiaddrConnection {
    const maConn: MultiaddrConnection = {
      ...stream,
      conn: stream,
      localAddr: Multiaddr(`/p2p/${this._peerInfo.id.toB58String()}`),
      remoteAddr: Multiaddr(`/p2p/${counterparty.toB58String()}`),
      async close(err?: Error) {
        if (err !== undefined) {
          error(err)
        }

        try {
          await connection.close()
        } catch (err) {
          error(err)
        }

        maConn.timeline.close = Date.now()
      },
      timeline: {
        open: Date.now(),
      },
    }

    return maConn
  }
}

export default TCP
