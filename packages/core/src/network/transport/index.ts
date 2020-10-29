import net from 'net'
import {AbortError} from 'abortable-iterator'
import type {Socket} from 'net'
import mafmt from 'mafmt'
import errCode from 'err-code'
import debug from 'debug'
import {socketToConn} from './socket-to-conn'
import libp2p from 'libp2p'
import Listener from './listener'
import {USE_WEBRTC, CODE_P2P} from './constants'
import Multiaddr from 'multiaddr'
import type {Upgrader, DialOptions, ConnHandler, MultiaddrConnection} from 'libp2p'
import type {Connection} from 'libp2p'
import chalk from 'chalk'
import {WebRTCUpgrader} from './webrtc'
import Relay from './relay'
import PeerId from 'peer-id'

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
  private _upgrader: Upgrader
  private relays?: Multiaddr[]
  private stunServers: Multiaddr[]
  private _relay: Relay
  // @ts-ignore
  private _webRTCUpgrader: WebRTCUpgrader
  private connHandler: ConnHandler
  private id: PeerId
  private multiaddrs: Multiaddr[]

  // ONLY FOR TESTING
  //@ts-ignore
  private _failIntentionallyOnWebRTC?: boolean
  //@ts-ignore
  private _timeoutIntentionallyOnWebRTC?: Promise<void>
  //@ts-ignore
  private _answerIntentionallyWithIncorrectMessages?: boolean
  // END ONLY FOR TESTING

  constructor({
    upgrader,
    libp2p,
    bootstrapServers,
    useWebRTC,
    failIntentionallyOnWebRTC,
    timeoutIntentionallyOnWebRTC,
    answerIntentionallyWithIncorrectMessages
  }: {
    upgrader: Upgrader
    libp2p: libp2p
    bootstrapServers?: Multiaddr[]
    useWebRTC?: boolean
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

    if (bootstrapServers?.length > 0) {
      this.relays = bootstrapServers.filter(
        (ma: Multiaddr) => ma !== undefined && libp2p.peerId.toB58String() !== ma.getPeerId()
      )

      this.stunServers = []
      for (let i = 0; i < this.relays.length; i++) {
        const opts = this.relays[i].toOptions()

        switch (opts.family) {
          case 'ipv6':
            // We do not use STUN for IPv6 for the moment
            break
          case 'ipv4':
            this.stunServers.push(this.relays[i])
            break
          default:
            throw Error('Invalid family')
        }
      }
    }

    this._timeoutIntentionallyOnWebRTC = timeoutIntentionallyOnWebRTC
    this._answerIntentionallyWithIncorrectMessages = answerIntentionallyWithIncorrectMessages
    this._failIntentionallyOnWebRTC = failIntentionallyOnWebRTC || false
    this._useWebRTC = useWebRTC === undefined ? USE_WEBRTC : useWebRTC
    this._upgrader = upgrader
    this.id = libp2p.peerId
    this.multiaddrs = libp2p.multiaddrs

    if (this._useWebRTC) {
      this._webRTCUpgrader = new WebRTCUpgrader({stunServers: this.stunServers})
    }

    this._relay = new Relay(libp2p, this.handleDelivery.bind(this))
    verbose(`Created TCP stack (Stun: ${this.stunServers?.map((x) => x.toString()).join(',')}`)
  }

  async handleDelivery(relayConn: MultiaddrConnection) {
    //verbose('handle delivery', connection.remoteAddr.toString(), counterparty.toB58String())
    let conn: Connection

    try {
      // @ts-ignore
      conn = await this._upgrader.upgradeInbound(relayConn)
    } catch (err) {
      error(`Could not upgrade relayed connection. Error was: ${err}`)
      return
    }

    this.connHandler?.(conn)
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
    if (
      // (this.relays == null || this.relays.some((pInfo) => ma.getPeerId() === pInfo.id.toB58String())) &&
      ['ip4', 'ip6', 'dns4', 'dns6'].includes(ma.protoNames()[0]) &&
      this.isRealisticAddress(ma)
    ) {
      try {
        verbose('attempting to dial directly', ma.toString())
        return await this.dialDirectly(ma, options)
      } catch (err) {
        if (
          (err.code != null && ['ECONNREFUSED', 'ECONNRESET', 'EPIPE'].includes(err.code)) ||
          err.type === 'aborted'
        ) {
          // expected case, continue
          error = err
        } else {
          // Unexpected error
          verbose(`Dial directly unexpected error ${err}`)
          throw err
        }
      }
    }

    if (this.relays === undefined) {
      throw Error(
        `Could not connect ${chalk.yellow(
          ma.toString()
        )} because we can't connect directly and we have no potential relays.${
          error != null ? ` Connection error was:\n${error}` : ''
        }`
      )
    }

    // Check whether we know some relays that we can use
    const potentialRelays = this.relays?.filter((r: Multiaddr) => r.getPeerId() !== ma.getPeerId())

    if (potentialRelays == null || potentialRelays.length == 0) {
      throw Error(
        `Destination ${chalk.yellow(
          ma.toString()
        )} cannot be accessed and directly and there is no other relay node known.${
          error != null ? ` Connection error was:\n${error}` : ''
        }`
      )
    }

    verbose('dialing with relay ', ma.toString())
    const conn = await this.dialWithRelay(ma, potentialRelays, options)
    log(`relayed connection established`)
    return conn
  }

  async dialWithRelay(ma: Multiaddr, relays: Multiaddr[], options?: DialOptions): Promise<Connection> {
    const relayConnection = await this._relay.establishRelayedConnection(ma, relays, options)
    return await this._upgrader.upgradeOutbound(relayConnection)
  }

  async dialDirectly(ma: Multiaddr, options?: DialOptions): Promise<Connection> {
    log(`[${chalk.blue(this.id.toB58String())}] dialing ${chalk.yellow(ma.toString())} directly`)
    const socket = await this._connect(ma, options)
    const maConn = socketToConn(socket, {remoteAddr: ma, signal: options?.signal})

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
      const cOpts = ma.toOptions()

      log('dialing %j', cOpts)
      const rawSocket = net.createConnection({
        host: cOpts.host,
        port: cOpts.port
      })

      const onError = (err: Error) => {
        verbose('Error connecting:', err)
        // ENETUNREACH
        // ECONNREFUSED
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
   * @param {function(Connection)} handler
   * @returns {Listener} A TCP listener
   */
  createListener(options: any, handler: (connection: Connection) => void): Listener {
    if (options == null) {
      this.connHandler = options
    } else {
      this.connHandler = handler
    }
    return new Listener(this.connHandler, this._upgrader, this.stunServers)
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

  /**
   * Filters unrealistic addresses
   * @param ma Multiaddr to check
   */
  private isRealisticAddress(ma: Multiaddr): boolean {
    if (ma.getPeerId() === this.id.toB58String()) {
      log('Tried to dial self, skipping.')
      return false
    }

    if (ma.protoNames()[0] === 'p2p') {
      return true
    }

    if (
      ['ip4', 'ip6', 'dns4', 'dns6'].includes(ma.protoNames()[0]) &&
      this.multiaddrs
        .map((ma: Multiaddr) => ma.nodeAddress())
        .filter(
          (x) =>
            x.address === ma.nodeAddress().address || // Same private network
            ma.nodeAddress().address === '127.0.0.1' // localhost
        )
        .filter((x) => x.port == ma.nodeAddress().port).length // Same port // Therefore dialing self.
    ) {
      log(`Tried to dial host on same network / port - aborting: ${ma.toString()}`)
      return false
    }

    return true
  }
}

export default TCP
