import Debug from 'debug'
import { CODE_DNS4, CODE_DNS6, CODE_IP4, CODE_IP6, CODE_P2P } from './constants.js'
import type { Multiaddr } from 'multiaddr'
import PeerId from 'peer-id'
import type Connection from 'libp2p-interfaces/src/connection/connection.js'
import type { Upgrader, Transport, MultiaddrConnection } from 'libp2p-interfaces/src/transport/types.js'
import type libp2p from 'libp2p'
import chalk from 'chalk'
import { TCPConnection, Listener } from './base/index.js'
import { Relay } from './relay/index.js'
import { Filter } from './filter.js'
import { Discovery } from './discovery.js'
// Do not type-check JSON files
// @ts-ignore
import { version } from '../package.json' assert { type: 'json' }

import type {
  PublicNodesEmitter,
  HoprConnectListeningOptions,
  HoprConnectDialOptions,
  HoprConnectOptions,
  HoprConnectTestingOptions
} from './types.js'

const DEBUG_PREFIX = 'hopr-connect'
const log = Debug(DEBUG_PREFIX)
const verbose = Debug(DEBUG_PREFIX.concat(':verbose'))
const warn = Debug(DEBUG_PREFIX.concat(':warn'))
const error = Debug(DEBUG_PREFIX.concat(':error'))

type HoprConnectConfig = {
  config?: HoprConnectOptions
  testing?: HoprConnectTestingOptions
}

/**
 * @class HoprConnect
 */
class HoprConnect implements Transport<HoprConnectDialOptions, HoprConnectListeningOptions> {
  get [Symbol.toStringTag]() {
    return 'HoprConnect'
  }

  public discovery: Discovery

  private _dialDirectly: HoprConnect['dialDirectly']
  private _upgradeOutbound: Upgrader['upgradeOutbound']
  private _upgradeInbound: Upgrader['upgradeInbound']

  private options: HoprConnectOptions
  private testingOptions: HoprConnectTestingOptions

  private _peerId: PeerId
  private relay: Relay
  private _addressFilter: Filter
  private _libp2p: libp2p

  constructor(
    opts: {
      upgrader: Upgrader
      libp2p: libp2p
    } & HoprConnectConfig
  ) {
    if (!opts.upgrader) {
      throw new Error('An upgrader must be provided. See https://github.com/libp2p/interface-transport#upgrader.')
    }

    if (!opts.libp2p) {
      throw new Error('Transport module needs access to libp2p.')
    }

    this.options = opts.config ?? {}
    this.testingOptions = opts.testing ?? {}

    this._peerId = opts.libp2p.peerId
    this._libp2p = opts.libp2p

    this._addressFilter = new Filter(this._peerId, this.options)

    this.discovery = new Discovery()

    this._upgradeOutbound = opts.upgrader.upgradeOutbound.bind(opts.upgrader)
    this._upgradeInbound = opts.upgrader.upgradeInbound.bind(opts.upgrader)

    this._dialDirectly = this.dialDirectly.bind(this)

    this.relay = new Relay(this._libp2p, this._dialDirectly, this.filter.bind(this), this.options, this.testingOptions)

    log(`HoprConnect: `, version)

    if (!!this.testingOptions.__noDirectConnections) {
      verbose(`DEBUG mode: always using relayed or WebRTC connections.`)

      const onConnection = this._libp2p.upgrader.onConnection

      // Simulated NAT:
      // If we don't allow direct connections (being a NATed node), then a connection
      // can happen if outgoing, i.e. by establishing a connection to someone else
      // we populate the address mapping of the router.
      // Or, if we get contacted by a relay to which we already have an *outgoing*
      // connection that gets reused.
      this._libp2p.upgrader.onConnection = (conn) => {
        log(`New connection:`)
        log(`remoteAddr: ${conn.remoteAddr.toString()}`)
        log(`remotePeer ${conn.remotePeer.toB58String()}`)
        log(`localAddr: ${conn.localAddr?.toString()}`)
        log(`remotePeer ${conn.localPeer.toB58String()}`)

        if (conn.remoteAddr.toString().startsWith(`/p2p/`)) {
          onConnection(conn)
          return
        }

        if (conn.stat.direction === 'outbound') {
          onConnection(conn)
          return
        }

        log(`closing due to NAT`)

        // Close the NATed connection as there is no need to keep
        // unused connections open.
        conn.close()
      }
    }

    if (!!this.testingOptions.__noWebRTCUpgrade) {
      verbose(`DEBUG mode: no WebRTC upgrade`)
    }

    if (!!this.testingOptions.__preferLocalAddresses) {
      verbose(`DEBUG mode: treat local addresses as public addresses.`)
    }
  }

  /**
   * Tries to establish a connection to the given destination
   * @param ma destination
   * @param options optional dial options
   * @returns An upgraded Connection
   */
  async dial(ma: Multiaddr, options: HoprConnectDialOptions = {}): Promise<Connection> {
    const maTuples = ma.tuples()

    // This works because destination peerId is for both address
    // types at the third place.
    // Other addresses are not supported.
    const destination = PeerId.createFromBytes((maTuples[2][1] as Uint8Array).slice(1))

    if (destination.equals(this._peerId)) {
      throw new Error(`Cannot dial ourself`)
    }

    switch (maTuples[0][0]) {
      case CODE_DNS4:
      case CODE_DNS6:
      case CODE_IP4:
      case CODE_IP6:
        return this.dialDirectly(ma, options)
      case CODE_P2P:
        const relay = PeerId.createFromBytes((maTuples[0][1] as Uint8Array).slice(1))

        return this.dialWithRelay(relay, destination, options)
      default:
        throw new Error(`Protocol not supported. Given address: ${ma.toString()}`)
    }
  }

  private onClose(): void {
    this.relay.stop()
  }

  private onListening(): void {
    this.relay.start()
  }

  /**
   * Creates a TCP listener. The provided `handler` function will be called
   * anytime a new incoming Connection has been successfully upgraded via
   * `upgrader.upgradeInbound`.
   * @param _handler
   * @returns A TCP listener
   */
  public createListener(_options: HoprConnectListeningOptions, _handler?: Function): Listener {
    return new Listener(
      this.onClose.bind(this),
      this.onListening.bind(this),
      this._dialDirectly,
      this._upgradeInbound,
      this._peerId,
      this.options,
      this.testingOptions,
      this._addressFilter,
      this.relay,
      this._libp2p
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
  public filter(multiaddrs: Multiaddr[]): Multiaddr[] {
    return (Array.isArray(multiaddrs) ? multiaddrs : [multiaddrs]).filter(
      this._addressFilter.filter.bind(this._addressFilter)
    )
  }

  /**
   * Attempts to establish a relayed connection to one of the given relays.
   * @param relay peerId of designated relay that we can use
   * @param destination peerId of destination
   * @param options optional dial options
   */
  private async dialWithRelay(
    relay: PeerId,
    destination: PeerId,
    options: HoprConnectDialOptions
  ): Promise<Connection> {
    log(
      `Attempting to dial ${chalk.yellow(`/p2p/${relay.toB58String()}/p2p-circuit/p2p/${destination.toB58String()}`)}`
    )

    let maConn = await this.relay.connect(relay, destination, options)

    if (maConn == undefined) {
      throw Error(`Could not establish relayed connection.`)
    }

    let conn: Connection

    try {
      conn = await this._upgradeOutbound(maConn)
      log(`Successfully established relayed connection to ${destination.toB58String()}`)
    } catch (err) {
      error(err)
      // libp2p needs this error to understand that this connection attempt failed but we
      // want to log it for debugging purposes
      throw err
    }

    return conn
  }

  /**
   * Attempts to establish a direct connection
   * @param ma destination
   * @param options optional dial options
   */
  public async dialDirectly(ma: Multiaddr, options?: HoprConnectDialOptions): Promise<Connection> {
    log(`Attempting to dial ${chalk.yellow(ma.toString())}`)

    const maConn = await TCPConnection.create(ma, this._peerId, options)

    verbose(
      `Establishing a direct connection to ${maConn.remoteAddr.toString()} was successful. Continuing with the handshake.`
    )

    const conn = await this._upgradeOutbound(maConn as MultiaddrConnection)

    // Assign various connection properties once we're sure that public keys match,
    // i.e. dialed node == desired destination

    // Set the SO_KEEPALIVE flag on socket to tell kernel to be more aggressive on keeping the connection up
    maConn.conn.setKeepAlive(true, 1000)

    maConn.conn.on('end', function () {
      log(`SOCKET END on connection to ${maConn.remoteAddr.toString()}:  other end of the socket sent a FIN packet`)
    })

    maConn.conn.on('timeout', function () {
      warn(`SOCKET TIMEOUT on connection to ${maConn.remoteAddr.toString()}`)
    })

    maConn.conn.on('error', function (e) {
      error(`SOCKET ERROR on connection to ${maConn.remoteAddr.toString()}: ' ${JSON.stringify(e)}`)
    })

    maConn.conn.on('close', (had_error) => {
      log(`SOCKET CLOSE on connection to ${maConn.remoteAddr.toString()}: error flag is ${had_error}`)
      // Don't call the disconnect handler if connection has been closed intentionally
      if (!maConn.closed && options && options.onDisconnect) {
        options.onDisconnect(ma)
      }
    })

    verbose(`Direct connection to ${maConn.remoteAddr.toString()} has been established successfully!`)

    return conn
  }
}

export type { PublicNodesEmitter, HoprConnectConfig, HoprConnectDialOptions, HoprConnectListeningOptions }
export { compareAddressesLocalMode, compareAddressesPublicMode } from './utils/index.js'

export default HoprConnect
