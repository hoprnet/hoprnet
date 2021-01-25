import mafmt from 'mafmt'
import debug from 'debug'
import Listener from './listener'
import { CODE_P2P, DELIVERY, USE_WEBRTC } from './constants'
import { AbortError } from 'abortable-iterator'
import type Multiaddr from 'multiaddr'
import PeerId from 'peer-id'
import type libp2p from 'libp2p'
import type { Dialer, Upgrader, DialOptions, ConnHandler, Handler, ConnectionManager } from 'libp2p'
import { Transport, Connection } from 'libp2p-interfaces'
import chalk from 'chalk'
import { TCPConnection } from './tcp'
import { WebRTCUpgrader } from './webrtc'
import Relay from './relay'
import { WebRTCConnection } from './webRTCConnection'
import type { RelayConnection } from './relayConnection'
import { Discovery } from './discovery'

const log = debug('hopr-connect')
const error = debug('hopr-connect:error')
const verbose = debug('hopr-connect:verbose')

/**
 * @class HoprConnect
 */
class HoprConnect implements Transport {
  get [Symbol.toStringTag]() {
    return 'HoprConnect'
  }

  public discovery: Discovery

  private __noDirectConnections?: boolean
  private __noWebRTCUpgrade?: boolean
  private _upgrader: Upgrader
  private _peerId: PeerId
  private _multiaddrs: Multiaddr[]
  private relays?: Multiaddr[]
  private stunServers?: Multiaddr[]
  private _relay: Relay
  private _connectionManager: ConnectionManager
  private _dialer: Dialer
  private _webRTCUpgrader?: WebRTCUpgrader
  private _interface?: string
  private connHandler?: ConnHandler

  constructor(opts: {
    upgrader: Upgrader
    libp2p: libp2p
    bootstrapServers?: Multiaddr[] | Multiaddr
    interface?: string
    __noDirectConnections?: boolean
    __noWebRTCUpgrade?: boolean
  }) {
    if (!opts.upgrader) {
      throw new Error('An upgrader must be provided. See https://github.com/libp2p/interface-transport#upgrader.')
    }

    if (!opts.libp2p) {
      throw new Error('Transport module needs access to libp2p.')
    }

    if (opts.bootstrapServers != undefined) {
      if (!Array.isArray(opts.bootstrapServers)) {
        opts.bootstrapServers = [opts.bootstrapServers]
      }

      for (const bs of opts.bootstrapServers) {
        const bsPeerId = bs.getPeerId()

        if (bsPeerId == undefined) {
          continue
        }

        if (opts.libp2p.peerId.equals(PeerId.createFromCID(bsPeerId))) {
          continue
        }

        const cOpts = bs.toOptions()

        if (this.relays == undefined) {
          this.relays = [bs]
        } else {
          this.relays.push(bs)
        }

        switch (cOpts.family) {
          case 'ipv6':
            // We do not use STUN for IPv6 for the moment
            break
          case 'ipv4':
            if (this.stunServers == undefined) {
              this.stunServers = [bs]
            } else {
              this.stunServers.push(bs)
            }
            break
          default:
            throw Error(`Invalid address family as STUN server. Got ${cOpts.family}`)
        }
      }
    }

    this._peerId = opts.libp2p.peerId
    this._multiaddrs = opts.libp2p.multiaddrs
    this._upgrader = opts.upgrader
    this._connectionManager = opts.libp2p.connectionManager
    this._dialer = opts.libp2p.dialer
    this._interface = opts.interface

    if (USE_WEBRTC) {
      this._webRTCUpgrader = new WebRTCUpgrader({ stunServers: this.stunServers })
    }

    this.discovery = new Discovery()

    opts.libp2p.handle(DELIVERY, this.handleIncoming.bind(this))

    this._relay = new Relay(opts.libp2p, this._webRTCUpgrader, opts.__noWebRTCUpgrade)

    // Used for testing
    this.__noDirectConnections = opts.__noDirectConnections
    this.__noWebRTCUpgrade = opts.__noWebRTCUpgrade

    try {
      const { version } = require('../package.json')

      log(`HoprConnect:`, version)
    } catch {}

    if (this.__noDirectConnections) {
      verbose(`DEBUG mode: always using relayed / WebRTC connections.`)
    }

    if (this.__noWebRTCUpgrade) {
      verbose(`DEBUG mode: no WebRTC upgrade`)
    }

    verbose(
      `Created ${this[Symbol.toStringTag]} stack (Stun: ${this.stunServers
        ?.map((server: Multiaddr) => server.toString())
        .join(',')})`
    )
  }

  /**
   * Tries to establish a connection to the given destination
   * @param ma destination
   * @param options optional dial options
   * @returns An upgraded Connection
   */
  async dial(ma: Multiaddr, options: DialOptions = {}): Promise<Connection> {
    if (options.signal?.aborted) {
      throw new AbortError()
    }

    switch (ma.protoNames()[0]) {
      case 'ip4':
      case 'ip6':
        if (!this.shouldAttemptDirectDial(ma)) {
          throw new AbortError()
        }

        return await this.dialDirectly(ma, options)
      case 'p2p':
        if (this.relays == undefined || this.relays.length == 0) {
          throw Error(
            `Could not connect to ${chalk.yellow(
              ma.toString()
            )}: Direct connection failed and there are no relays known.`
          )
        }

        verbose('dialing with relay ', ma.toString())
        return await this.dialWithRelay(ma, this.relays, options)
      default:
        throw new AbortError(`Protocol not supported`)
    }
  }

  /**
   * Creates a TCP listener. The provided `handler` function will be called
   * anytime a new incoming Connection has been successfully upgraded via
   * `upgrader.upgradeInbound`.
   * @param handler
   * @returns A TCP listener
   */
  createListener(options: any | undefined, handler: (connection: Connection) => void): Listener {
    if (arguments.length == 1 && typeof options === 'function') {
      this.connHandler = options
    } else {
      this.connHandler = handler
    }
    return new Listener(this.connHandler, this._upgrader, this.stunServers, this._peerId, this._interface)
  }

  /**
   * Takes a list of Multiaddrs and returns those addrs that we can use.
   * @example
   * Multiaddr(`/ip4/127.0.0.1/tcp/0`) // working
   * Multiaddr(`/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg`) // working
   * @param multiaddrs
   * @returns applicable Multiaddrs
   */
  filter(multiaddrs: Multiaddr[]): Multiaddr[] {
    return (Array.isArray(multiaddrs) ? multiaddrs : [multiaddrs]).filter(
      (ma: Multiaddr) => mafmt.TCP.matches(ma.decapsulateCode(CODE_P2P)) || mafmt.P2P.matches(ma)
    )
  }

  /**
   * Attempts to establish a relayed connection to one of the given relays.
   * @param ma destination
   * @param relays potential relays that we can use
   * @param options optional dial options
   */
  private async dialWithRelay(ma: Multiaddr, relays: Multiaddr[], options?: DialOptions): Promise<Connection> {
    let conn = await this._relay.establishRelayedConnection(ma, relays, this.onReconnect.bind(this), options)

    return await this._upgrader.upgradeOutbound(conn)
  }

  /**
   * Attempts to establish a direct connection
   * @param ma destination
   * @param options optional dial options
   */
  private async dialDirectly(ma: Multiaddr, options?: DialOptions): Promise<Connection> {
    log(`Attempting to dial ${chalk.yellow(ma.toString())} directly`)
    const maConn = await TCPConnection.create(ma, options)

    verbose(
      `Establishing a direct connection to ${maConn.remoteAddr.toString()} was successful. Continuing with the handshake.`
    )
    const conn = await this._upgrader.upgradeOutbound(maConn)
    verbose('outbound direct connection %s upgraded', maConn.remoteAddr)
    return conn
  }

  /**
   * Dialed once a reconnect happens
   * @param newStream new relayed connection
   * @param counterparty counterparty of the relayed connection
   */
  private async onReconnect(this: HoprConnect, newStream: RelayConnection, counterparty: PeerId): Promise<void> {
    log(`####### inside reconnect #######`)

    let newConn: Connection

    try {
      if (this._webRTCUpgrader != undefined) {
        newConn = await this._upgrader.upgradeInbound(
          new WebRTCConnection(
            {
              conn: newStream,
              self: this._peerId,
              counterparty,
              channel: newStream.webRTC!.channel
            },
            {
              __noWebRTCUpgrade: this.__noWebRTCUpgrade
            }
          )
        )
      } else {
        newConn = await this._upgrader.upgradeInbound(newStream)
      }
    } catch (err) {
      error(err)
      return
    }

    this._dialer._pendingDials[counterparty.toB58String()]?.destroy()
    this._connectionManager.connections.set(counterparty.toB58String(), [newConn])

    this.connHandler?.(newConn)
  }

  /**
   * Called once a relayed connection is establishing
   * @param handler handles the relayed connection
   */
  private async handleIncoming(this: HoprConnect, handler: Handler): Promise<void> {
    let newConn: Connection

    try {
      const relayConnection = await this._relay.handleRelayConnection(handler, this.onReconnect.bind(this))

      if (relayConnection == undefined) {
        return
      }

      newConn = await this._upgrader.upgradeInbound(relayConnection)
    } catch (err) {
      error(`Could not upgrade relayed connection. Error was: ${err}`)
      return
    }

    this.discovery._peerDiscovered(newConn.remotePeer, [newConn.remoteAddr])

    this.connHandler?.(newConn)
  }

  /**
   * Return true if we should attempt a direct dial.
   * @param ma Multiaddr to check
   */
  private shouldAttemptDirectDial(ma: Multiaddr): boolean {
    // Forces node to only use relayed connection and
    // don't try a direct dial attempt.
    // Used for testing
    if (
      this.__noDirectConnections &&
      (this.relays == undefined || !this.relays.some((mAddr: Multiaddr) => ma.getPeerId() === mAddr.getPeerId()))
    ) {
      return false
    }
    // uncommenting next line forces our node to use a relayed connection to any node execpt for the bootstrap server
    let protoNames = ma.protoNames()
    if (!['ip4', 'ip6', 'dns4', 'dns6'].includes(protoNames[0])) {
      // We cannot call other protocols directly
      return false
    }

    let cOpts = ma.toOptions()

    for (const mAddr of this._multiaddrs) {
      const ownOpts = mAddr.toOptions()

      if (ownOpts.host === cOpts.host && ownOpts.port == cOpts.port) {
        return false
      }
    }

    return true
  }
}

export default HoprConnect
