import Debug from 'debug'
import { CODE_IP4, CODE_IP6, CODE_P2P, USE_WEBRTC } from './constants'
import { AbortError } from 'abortable-iterator'
import type { Multiaddr } from 'multiaddr'
import PeerId from 'peer-id'
import type { Upgrader } from 'libp2p-interfaces/src/transport/types'
import type { default as libp2p, Connection } from 'libp2p'
import { Transport } from 'libp2p-interfaces/src/transport/types'
import chalk from 'chalk'
import { TCPConnection, Listener } from './base'
import { WebRTCUpgrader } from './webrtc'
import { Relay } from './relay'
import { Discovery } from './discovery'
import { Filter } from './filter'
import { dialHelper } from './utils'

import type { PublicNodesEmitter, PeerStoreType, DialOptions } from './types'

const log = Debug('hopr-connect')
const verbose = Debug('hopr-connect:verbose')

export type HoprConnectOptions = {
  publicNodes?: PublicNodesEmitter
  initialNodes?: PeerStoreType[]
  interface?: string
  __noDirectConnections?: boolean
  __noWebRTCUpgrade?: boolean
  maxRelayedConnections?: number
  __relayFreeTimeout?: number
}

/**
 * @class HoprConnect
 */
class HoprConnect implements Transport<DialOptions, any> {
  get [Symbol.toStringTag]() {
    return 'HoprConnect'
  }

  public discovery: Discovery

  private publicNodes?: PublicNodesEmitter
  private initialNodes?: PeerStoreType[]

  private relayPeerIds?: Set<string>

  private __noDirectConnections?: boolean
  private __noWebRTCUpgrade?: boolean
  private _upgrader: Upgrader
  private _peerId: PeerId
  private relay: Relay
  private _webRTCUpgrader?: WebRTCUpgrader
  private _interface?: string
  private _addressFilter: Filter
  private _libp2p: libp2p

  private connHandler: ((conn: Connection) => void) | undefined

  constructor(
    opts: {
      upgrader: Upgrader
      libp2p: libp2p
    } & HoprConnectOptions
  ) {
    if (!opts.upgrader) {
      throw new Error('An upgrader must be provided. See https://github.com/libp2p/interface-transport#upgrader.')
    }

    if (!opts.libp2p) {
      throw new Error('Transport module needs access to libp2p.')
    }

    this.publicNodes = opts.publicNodes
    this.initialNodes = opts.initialNodes

    this._peerId = opts.libp2p.peerId

    // @TODO only store references to needed parts of libp2p
    this._libp2p = opts.libp2p

    this._addressFilter = new Filter(this._peerId)

    this._upgrader = opts.upgrader
    this._interface = opts.interface

    if (USE_WEBRTC) {
      this._webRTCUpgrader = new WebRTCUpgrader(this.publicNodes, this.initialNodes)
    }

    this.discovery = new Discovery()

    this.relay = new Relay(
      (peer: PeerId, protocol: string, options: { timeout: number } | DialOptions) =>
        dialHelper(opts.libp2p, peer, protocol, options as any) as any,
      opts.libp2p.dialer,
      opts.libp2p.connectionManager,
      opts.libp2p.handle.bind(opts.libp2p),
      this._peerId,
      this._upgrader,
      this.connHandler,
      this._webRTCUpgrader,
      opts.__noWebRTCUpgrade,
      opts.maxRelayedConnections,
      opts.__relayFreeTimeout
    )

    // Used for testing
    this.__noDirectConnections = opts.__noDirectConnections
    this.__noWebRTCUpgrade = opts.__noWebRTCUpgrade

    try {
      const { version } = require('../package.json')

      log(`HoprConnect: `, version)
    } catch {
      console.error(`Cannot find package.json to load version tag. Exitting.`)
      return
    }

    if (this.__noDirectConnections) {
      // Whenever we don't allow direct connections, we need to store
      // the known relays and make sure that we allow direct connections
      // to them.
      this.relayPeerIds = new Set<string>()

      for (const initialNode of this.initialNodes ?? []) {
        this.relayPeerIds ??= new Set<string>()
        this.relayPeerIds.add(initialNode.id.toB58String())
      }

      this.publicNodes?.on('addPublicNode', (peer: PeerStoreType) => {
        this.relayPeerIds ??= new Set<string>()
        this.relayPeerIds.add(peer.id.toB58String())
      })

      verbose(`DEBUG mode: always using relayed / WebRTC connections.`)
    }

    if (this.__noWebRTCUpgrade) {
      verbose(`DEBUG mode: no WebRTC upgrade`)
    }
  }

  /**
   * Removes a relay from the list of usable relays
   * @TODO to be implemented
   */
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
      this.publicNodes,
      this.initialNodes,
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
  private async dialWithRelay(relay: PeerId, destination: PeerId, options: DialOptions): Promise<Connection> {
    let conn = await this.relay.connect(relay, destination, options)

    if (conn == undefined) {
      throw Error(`Could not establish relayed connection.`)
    }

    return await this._upgrader.upgradeOutbound(conn as any)
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
    return await this._upgrader.upgradeOutbound(maConn as any)
  }

  /**
   * Return true if we should attempt a direct dial.
   * @param ma Multiaddr to check
   */
  private shouldAttemptDirectDial(ma: Multiaddr): boolean {
    const maPeerId = ma.getPeerId()
    if (
      // Forces the node to only use relayed connections and
      // don't try a direct dial attempt.
      // @dev Used for testing
      this.__noDirectConnections &&
      (this.relayPeerIds == undefined ||
        this.relayPeerIds.size == 0 ||
        (maPeerId != null && !this.relayPeerIds.has(maPeerId)))
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
