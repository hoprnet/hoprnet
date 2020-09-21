import net from 'net'
import { AbortError } from 'abortable-iterator'
import type { Socket } from 'net'
import mafmt from 'mafmt'
import errCode from 'err-code'
import debug from 'debug'
import { socketToConn } from './socket-to-conn'
import myHandshake from './handshake'
// @ts-ignore
import libp2p = require('libp2p')
import { createListener, Listener } from './listener'
import { multiaddrToNetConfig } from './utils'
import { USE_WEBRTC, CODE_P2P, USE_OWN_STUN_SERVERS } from './constants'
import Multiaddr from 'multiaddr'
import PeerInfo from 'peer-info'
import PeerId from 'peer-id'
import pipe from 'it-pipe'
import type { Connection, Upgrader, DialOptions, ConnHandler, Handler, Stream, MultiaddrConnection } from './types'
import chalk from 'chalk'
import pushable, { Pushable } from 'it-pushable'
import upgradeToWebRTC from './webrtc'
import Relay from './relay'

const log = debug('hopr-core:transport')
const error = debug('hopr-core:transport:error')
const verbose = debug('hopr-core:verbose:transport')

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
  private _upgrader: Upgrader
  private _peerInfo: PeerInfo
  private _handle: (protocols: string[] | string, handler: (connection: Handler) => void) => void
  private relays?: PeerInfo[]
  private stunServers: { urls: string }[]
  private _relay: Relay
  private connHandler: ConnHandler

  // ONLY FOR TESTING
  private _failIntentionallyOnWebRTC?: boolean
  private _timeoutIntentionallyOnWebRTC?: Promise<void>
  private _answerIntentionallyWithIncorrectMessages?: boolean
  // END ONLY FOR TESTING

  constructor({
    upgrader,
    libp2p,
    bootstrapServers,
    useWebRTC,
    useOwnStunServers,
    failIntentionallyOnWebRTC,
    timeoutIntentionallyOnWebRTC,
    answerIntentionallyWithIncorrectMessages,
  }: {
    upgrader: Upgrader
    libp2p: libp2p
    bootstrapServers?: PeerInfo[]
    useWebRTC?: boolean
    useOwnStunServers?: boolean
    failIntentionallyOnWebRTC?: boolean
    timeoutIntentionallyOnWebRTC?: Promise<void>
    answerIntentionallyWithIncorrectMessages?: boolean
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
    this._answerIntentionallyWithIncorrectMessages = answerIntentionallyWithIncorrectMessages
    this._failIntentionallyOnWebRTC = failIntentionallyOnWebRTC || false
    this._useOwnStunServers = useOwnStunServers === undefined ? USE_OWN_STUN_SERVERS : useOwnStunServers
    this._useWebRTC = useWebRTC === undefined ? USE_WEBRTC : useWebRTC
    this._handle = libp2p.handle.bind(libp2p)
    this._peerInfo = libp2p.peerInfo
    this._upgrader = upgrader

    this._relay = new Relay(libp2p, this.handleDelivery.bind(this))
    verbose('Created TCP stack', this.stunServers)
  }

  async handleDelivery({ stream, connection, counterparty }: Handler & { counterparty: PeerId }) {
    verbose('handle delivery', connection.remoteAddr, counterparty.id)
    let conn: Connection

    let webRTCsendBuffer: Pushable<Uint8Array>
    let webRTCrecvBuffer: Pushable<Uint8Array>

    let socket: Promise<net.Socket>

    if (this._useWebRTC) {
      webRTCsendBuffer = pushable<Uint8Array>()
      webRTCrecvBuffer = pushable<Uint8Array>()

      verbose('attempting to upgrade to webRTC from a delivery', connection.remoteAddr)
      socket = upgradeToWebRTC(webRTCsendBuffer, webRTCrecvBuffer, {
        _timeoutIntentionallyOnWebRTC: this._timeoutIntentionallyOnWebRTC,
        _failIntentionallyOnWebRTC: this._failIntentionallyOnWebRTC,
        _answerIntentionallyWithIncorrectMessages: this._answerIntentionallyWithIncorrectMessages,
      })
    }

    const myStream = myHandshake(webRTCsendBuffer, webRTCrecvBuffer)

    pipe(
      // prettier-ignore
      stream.source,
      myStream.webRtcStream.source
    )

    pipe(
      // prettier-ignore
      myStream.webRtcStream.sink,
      stream.sink
    )

    if (this._useWebRTC) {
      const addr = counterparty.toB58String()
      try {
        let _socket = await socket

        webRTCrecvBuffer.end()
        webRTCsendBuffer.end()
        verbose('upgraded to webRTC, now attempting upgrade to direct conn', addr)

        conn = await this._upgrader.upgradeInbound(
          socketToConn(_socket, {
            remoteAddr: Multiaddr(`/p2p/${addr}`),
            localAddr: Multiaddr(`/p2p/${this._peerInfo.id.toB58String()}`),
          })
        )

        verbose('Established a direct webRTC connection')
      } catch (err) {
        verbose(`error while upgrading to webrtc direct connection ${err}: ${addr}`)

        webRTCrecvBuffer.end()
        webRTCsendBuffer.end()

        verbose('falling back to relayed connection')
        conn = await this._upgrader.upgradeInbound(
          this.relayToConn({
            stream: myStream.relayStream,
            counterparty,
            connection,
          })
        )
      }
    } else {
      try {
        conn = await this._upgrader.upgradeInbound(
          this.relayToConn({
            stream: myStream.relayStream,
            counterparty,
            connection,
          })
        )
        verbose('Established a relayed webRTC connection')
      } catch (err) {
        verbose('Error upgrading to a relayed webRTC connection', err)
        //error(err)
        return
      }
    }

    this.connHandler?.(conn)
  }

  private filterUnrealisticAddresses(ma: Multiaddr): boolean {
    if (ma.getPeerId() === this._peerInfo.id.toB58String()) {
      log('Tried to dial self, skipping.')
      return false
    }

    if (
      this._peerInfo.multiaddrs
        .toArray()
        .map((x) => x.nodeAddress())
        .filter(
          (x) =>
            x.address == ma.nodeAddress().address || // Same private network
            ma.nodeAddress().address == '127.0.0.1' // localhost
        )
        .filter((x) => x.port == ma.nodeAddress().port).length // Same port // Therefore dialing self.
    ) {
      log(`Tried to dial host on same network / port - aborting: ${ma.toString()}`)
      return false
    }

    return true
  }

  /**
   * @async
   * @param {Multiaddr} ma
   * @param {object} options
   * @param {AbortSignal} options.signal Used to abort dial requests
   * @returns {Connection} An upgraded Connection
   */
  async dial(ma: Multiaddr, options?: DialOptions): Promise<Connection> {
    if (!this.filterUnrealisticAddresses(ma)) {
      return new Promise((r, x) => x(new Error('Filtering unrealistic address')))
    }

    options = options || {}

    let error: Error
    if (['ip4', 'ip6', 'dns4', 'dns6'].includes(ma.protoNames()[0])) {
      try {
        verbose('attempting to dial directly', ma)
        return await this.dialDirectly(ma, options)
      } catch (err) {
        if (err.type === 'timeout') {
          // expected case, continue
          error = err
        } else {
          // Unexpected error, ie:
          // type === aborted
          verbose(`Dial directly unexpected error ${err}`)
          throw err
        }
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

    verbose('dialing with relay ', ma)
    return await this.dialWithRelay(ma, potentialRelays, options)
  }

  async dialWithRelay(ma: Multiaddr, relays: PeerInfo[], options?: DialOptions): Promise<Connection> {
    let webRTCsendBuffer: Pushable<Uint8Array>
    let webRTCrecvBuffer: Pushable<Uint8Array>

    const destination = PeerId.createFromCID(ma.getPeerId())

    const relayConnection = await this._relay.establishRelayedConnection(ma, relays, options)

    let conn: Connection

    if (options?.signal?.aborted) {
      try {
        await relayConnection.connection.close()
      } catch (err) {
        error(err)
      }

      throw new AbortError()
    }

    let socket: Promise<net.Socket>

    if (this._useWebRTC) {
      webRTCsendBuffer = pushable<Uint8Array>()
      webRTCrecvBuffer = pushable<Uint8Array>()

      verbose('attempting to upgrade a relay dial to webRTC')
      socket = upgradeToWebRTC(webRTCsendBuffer, webRTCrecvBuffer, {
        initiator: true,
        _timeoutIntentionallyOnWebRTC: this._timeoutIntentionallyOnWebRTC,
        _failIntentionallyOnWebRTC: this._failIntentionallyOnWebRTC,
        _answerIntentionallyWithIncorrectMessages: this._answerIntentionallyWithIncorrectMessages,
      })
    }

    const stream = myHandshake(webRTCsendBuffer, webRTCrecvBuffer, { signal: options?.signal })

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
        let _socket = await socket

        webRTCsendBuffer.end()
        webRTCrecvBuffer.end()

        conn = await this._upgrader.upgradeOutbound(
          socketToConn(_socket, {
            signal: options.signal,
            remoteAddr: Multiaddr(`/p2p/${destination.toB58String()}`),
            localAddr: Multiaddr(`/p2p/${this._peerInfo.id.toB58String()}`),
          })
        )
      } catch (err) {
        error(`error while dialling: ${err}`)
        webRTCsendBuffer.end()
        webRTCrecvBuffer.end()

        conn = await this._upgrader.upgradeOutbound(
          this.relayToConn({
            stream: stream.relayStream,
            counterparty: destination,
            connection: relayConnection.connection,
          })
        )
      }
    } else {
      try {
        conn = await this._upgrader.upgradeOutbound(
          this.relayToConn({
            stream: stream.relayStream,
            counterparty: destination,
            connection: relayConnection.connection,
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
    log(`[${chalk.blue(this._peerInfo.id.toB58String())}] dialing ${chalk.yellow(ma.toString())} directly`)
    const socket = await this._connect(ma, options)
    const maConn = socketToConn(socket, { remoteAddr: ma, signal: options?.signal })

    log('new outbound direct connection %s', maConn.remoteAddr)
    const conn = await this._upgrader.upgradeOutbound(maConn)
    log('outbound direct connection %s upgraded', maConn.remoteAddr)
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
        verbose('Error connecting:', err)
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
    verbose('filtering multiaddrs')
    return multiaddrs.filter((ma: Multiaddr) => {
      return mafmt.TCP.matches(ma.decapsulateCode(CODE_P2P)) || mafmt.P2P.matches(ma)
    })
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
