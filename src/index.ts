/// <reference path="./@types/libp2p.ts" />

import debug from 'debug'
import { CODE_IP4, CODE_IP6, CODE_P2P, USE_WEBRTC } from './constants'
import { AbortError } from 'abortable-iterator'
import type { Multiaddr } from 'multiaddr'
import PeerId from 'peer-id'
import type { Upgrader, DialOptions, ConnHandler, default as libp2p } from 'libp2p'
import { Transport, Connection } from 'libp2p-interfaces'
import chalk from 'chalk'
import { TCPConnection, Listener } from './base'
import { WebRTCUpgrader } from './webrtc'
import { Relay } from './relay'
import { Discovery } from './discovery'
import { Filter } from './filter'
import { dialHelper } from './utils'

const log = debug('hopr-connect')
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
  private relays?: Multiaddr[]
  private stunServers?: Multiaddr[]
  private _relay: Relay
  private _webRTCUpgrader?: WebRTCUpgrader
  private _interface?: string
  private _addressFilter: Filter
  private _libp2p: libp2p

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
          case 6:
            // We do not use STUN for IPv6 for the moment
            break
          case 4:
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

    // @TODO only store references to needed parts of libp2p
    this._libp2p = opts.libp2p

    this._addressFilter = new Filter(this._peerId)

    this._upgrader = opts.upgrader
    this._interface = opts.interface

    if (USE_WEBRTC) {
      this._webRTCUpgrader = new WebRTCUpgrader({ stunServers: this.stunServers })
    }

    this.discovery = new Discovery()

    this._relay = new Relay(
      (peer: PeerId, protocol: string, options: { timeout: number } | { signal: AbortSignal }) =>
        dialHelper(opts.libp2p, peer, protocol, options as any),
      opts.libp2p.dialer,
      opts.libp2p.connectionManager,
      opts.libp2p.handle.bind(opts.libp2p),
      this._peerId,
      this._upgrader,
      this.connHandler,
      this._webRTCUpgrader,
      opts.__noWebRTCUpgrade
    )

    // Used for testing
    this.__noDirectConnections = opts.__noDirectConnections
    this.__noWebRTCUpgrade = opts.__noWebRTCUpgrade

    try {
      const { version } = require('../package.json')

      log(`HoprConnect:`, version)
    } catch {
      console.error(`Cannot find package.json to load version tag. Exitting.`)
      return
    }

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

  addRelay() {}

  removeRelay() {}

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

    log(`Attempting to dial ${chalk.yellow(ma.toString())}`)

    const maTuples = ma.tuples()

    // This works because destination peerId is for both address
    // types at the third place.
    // Other addresses are not supported.
    const destination = PeerId.createFromBytes((maTuples[2][1] as Uint8Array).slice(1))

    if (destination.equals(this._peerId)) {
      throw new AbortError(`Cannot dial ourself`)
    }

    switch (maTuples[0][0]) {
      case CODE_IP4:
      case CODE_IP6:
        if (!this.shouldAttemptDirectDial(ma)) {
          throw new AbortError()
        }

        return await this.dialDirectly(ma, options)
      case CODE_P2P:
        const relay = PeerId.createFromBytes((maTuples[0][1] as Uint8Array).slice(1))

        return await this.dialWithRelay(relay, destination, options)
      default:
        throw new AbortError(`Protocol not supported. Given address: ${ma.toString()}`)
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
    return new Listener(
      this.connHandler,
      this._upgrader,
      this.stunServers,
      this.stunServers, // use STUN servers as relays
      this._peerId,
      this._interface
    )
  }

  /**
   * Takes a list of Multiaddrs and returns those addrs that we can use.
   * @example
   * new Multiaddr(`/ip4/127.0.0.1/tcp/0/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg`) // working
   * new Multiaddr(`/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg/p2p-circuit/p2p/16Uiu2HAkyvdVZtG8btak5SLrxP31npfJo6maopj8xwx5XQhKfspb`) // working
   * @param multiaddrs
   * @returns applicable Multiaddrs
   */
  filter(multiaddrs: Multiaddr[]): Multiaddr[] {
    if (this._libp2p.isStarted() && !this._addressFilter.addrsSet) {
      // Attaches addresses to AddressFilter
      // @TODO implement this in a cleaner way
      try {
        const addrs = this._libp2p.transportManager.getAddrs()
        this._addressFilter.setAddrs(addrs, this._libp2p.addressManager.getListenAddrs())
      } catch {}
    }
    return (Array.isArray(multiaddrs) ? multiaddrs : [multiaddrs]).filter(
      this._addressFilter.filter.bind(this._addressFilter)
    )
  }

  /**
   * Attempts to establish a relayed connection to one of the given relays.
   * @param ma destination
   * @param relays potential relays that we can use
   * @param options optional dial options
   */
  private async dialWithRelay(relay: PeerId, destination: PeerId, options?: DialOptions): Promise<Connection> {
    let conn = await this._relay.connect(relay, destination, options)

    if (conn == undefined) {
      throw Error(`Could not establish relayed connection.`)
    }

    return await this._upgrader.upgradeOutbound(conn)
  }

  /**
   * Attempts to establish a direct connection
   * @param ma destination
   * @param options optional dial options
   */
  private async dialDirectly(ma: Multiaddr, options?: DialOptions): Promise<Connection> {
    const maConn = await TCPConnection.create(ma, this._peerId, options)

    verbose(
      `Establishing a direct connection to ${maConn.remoteAddr.toString()} was successful. Continuing with the handshake.`
    )
    return await this._upgrader.upgradeOutbound(maConn)
  }

  /**
   * Return true if we should attempt a direct dial.
   * @param ma Multiaddr to check
   */
  private shouldAttemptDirectDial(ma: Multiaddr): boolean {
    if (
      // Forces the node to only use relayed connections and
      // don't try a direct dial attempts.
      // @dev Used for testing
      this.__noDirectConnections &&
      (this.relays == undefined || !this.relays.some((mAddr: Multiaddr) => ma.getPeerId() === mAddr.getPeerId()))
    ) {
      return false
    }

    let protoNames = ma.protoNames()
    if (!['ip4', 'ip6', 'dns4', 'dns6'].includes(protoNames[0])) {
      // We cannot call other protocols directly
      return false
    }

    return true
  }
}

export { HoprConnect }
