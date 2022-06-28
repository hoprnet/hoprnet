import type { Libp2p } from 'libp2p'
import type { PeerId } from '@libp2p/interface-peer-id'
import type { Connection, ProtocolStream, MultiaddrConnection } from '@libp2p/interface-connection'
import type { Multiaddr } from '@multiformats/multiaddr'
import type { Address } from '@libp2p/interface-peer-store'
import type { Upgrader } from '@libp2p/interface-transport'
import { peerIdFromString } from '@libp2p/peer-id'

import type HoprConnect from '../index.js'

import type { Stream, HoprConnectOptions, HoprConnectDialOptions, HoprConnectTestingOptions } from '../types.js'

import debug from 'debug'

import { WebRTCUpgrader, WebRTCConnection } from '../webrtc/index.js'
import chalk from 'chalk'
import { RELAY_PROTCOL, DELIVERY_PROTOCOL, CODE_P2P, OK, CAN_RELAY_PROTCOL } from '../constants.js'
import { RelayConnection } from './connection.js'
import { RelayHandshake, RelayHandshakeMessage } from './handshake.js'
import { RelayState } from './state.js'
import { createRelayerKey, randomInteger, retimer, tryExistingConnections } from '@hoprnet/hopr-utils'

import { attemptClose } from '../utils/index.js'

const DEBUG_PREFIX = 'hopr-connect:relay'
const DEFAULT_MAX_RELAYED_CONNECTIONS = 10

const log = debug(DEBUG_PREFIX)
const error = debug(DEBUG_PREFIX.concat(':error'))
const verbose = debug(DEBUG_PREFIX.concat(':verbose'))

type ConnResult = ProtocolStream & {
  conn: Connection
}

// Specify which libp2p methods this class uses
// such that Typescript fails to build if anything changes
type ReducedAddressBook = {
  get: (peerId: PeerId) => ReturnType<Libp2p['peerStore']['addressBook']['get']>
}
type ReducedPeerStore = {
  peerStore: {
    addressBook: ReducedAddressBook
  }
}
type ReducedDHT = { contentRouting: Pick<Libp2p['contentRouting'], 'provide'> }
type ReducedDialer = { dialer: Pick<Libp2p['dialer'], '_pendingDials'> }
type ReducedUpgrader = { upgrader: Pick<Upgrader, 'upgradeInbound' | 'upgradeOutbound'> }
type ReducedConnectionManager = { connectionManager: Pick<Libp2p['connectionManager'], 'getAll' | 'onDisconnect'> }
type ReducedLibp2p = ReducedPeerStore &
  ReducedDialer &
  ReducedConnectionManager &
  ReducedUpgrader & {
    peerId: PeerId
    handle: Libp2p['handle']
  } & ReducedDHT

/**
 * API interface for relayed connections
 */
class Relay {
  private relayState: RelayState
  private webRTCUpgrader: WebRTCUpgrader
  private usedRelays: PeerId[]

  private _onReconnect: Relay['onReconnect'] | undefined
  private _onDelivery: Relay['onDelivery'] | undefined
  private _onRelay: Relay['onRelay'] | undefined
  private _onCanRelay: Relay['onCanRelay'] | undefined
  private _dialNodeDirectly: Relay['dialNodeDirectly'] | undefined

  private stopKeepAlive: (() => void) | undefined
  private connectedToRelays: Set<string>

  constructor(
    public libp2p: ReducedLibp2p,
    private dialDirectly: HoprConnect['dialDirectly'],
    private filter: HoprConnect['filter'],
    private options: HoprConnectOptions,
    private testingOptions: HoprConnectTestingOptions
  ) {
    log(`relay testing options`, testingOptions)
    this.relayState = new RelayState()

    this.options.maxRelayedConnections ??= DEFAULT_MAX_RELAYED_CONNECTIONS

    this.webRTCUpgrader = new WebRTCUpgrader(this.options)

    // Stores all relays that we announce to other nodes
    // to make sure we don't close these connections
    this.usedRelays = []

    // Gathers relay peer IDs the node connected
    this.connectedToRelays = new Set()
  }

  /**
   * Assigns the event listeners to the constructed object.
   * @dev Must not happen in the constructor because `this` is not ready
   *      at that point in time
   */
  async start(): Promise<void> {
    this._onReconnect = this.onReconnect.bind(this)
    this._onRelay = this.onRelay.bind(this)
    this._onDelivery = this.onDelivery.bind(this)
    this._onCanRelay = this.onCanRelay.bind(this)

    this._dialNodeDirectly = this.dialNodeDirectly.bind(this)

    await this.libp2p.handle(DELIVERY_PROTOCOL(this.options.environment), this._onDelivery)
    await this.libp2p.handle(RELAY_PROTCOL(this.options.environment), this._onRelay)
    await this.libp2p.handle(CAN_RELAY_PROTCOL(this.options.environment), this._onCanRelay)

    this.webRTCUpgrader.start()

    // Periodic function that prints relay connections (and will also do pings in future)
    const periodicKeepAlive = async function (this: Relay) {
      try {
        await this.keepAliveRelayConnection()
      } catch (err) {
        log('Fatal error during periodic keep-alive of relay connections', err)
      }
    }.bind(this)

    this.stopKeepAlive = retimer(
      periodicKeepAlive,
      // TODO: Make these values configurable
      () => randomInteger(10000, 10000 + 3000)
    )
  }

  setUsedRelays(peers: PeerId[]) {
    log(`set used relays`, peers)
    this.usedRelays = peers
  }

  protected async keepAliveRelayConnection(): Promise<void> {
    // TODO: perform ping as well, right now just prints out connection info
    if (this.relayState.relayedConnectionCount() > 0) {
      log(`Current relay connections: `)
      await this.relayState.forEach(async (dst) => log(`- ${dst}`))
    }

    log(`Currently tracked connections to relays: `)
    this.connectedToRelays.forEach((relayPeerId) => {
      const countConns = this.libp2p.connectionManager.getAll(peerIdFromString(relayPeerId)).length
      log(`- ${relayPeerId}: ${countConns} connection${countConns == 1 ? '' : 's'}`)
    })
  }

  /**
   * Unassigns event listeners
   */
  stop(): void {
    this.stopKeepAlive?.()
    this.connectedToRelays.clear()
    this.webRTCUpgrader.stop()
  }

  /**
   * Attempts to connect to `destination` by using `relay` as a relay
   * @param relay relay to use
   * @param destination destination to connect to
   * @param options options, e.g. timeout
   * @returns a connection object if possible, otherwise undefined
   */
  public async connect(
    relay: PeerId,
    destination: PeerId,
    options?: HoprConnectDialOptions
  ): Promise<MultiaddrConnection | undefined> {
    const baseConnection = await this.dialNodeDirectly(relay, RELAY_PROTCOL(this.options.environment), {
      signal: options?.signal
    }).catch(error)

    if (baseConnection == undefined) {
      error(
        `Cannot establish a connection to ${chalk.green(destination.toString())} because relay ${chalk.green(
          relay.toString()
        )} is not reachable`
      )
      return
    }

    const shaker = new RelayHandshake(baseConnection.stream as Stream, this.options)

    const handshakeResult = await shaker.initiate(relay, destination)

    if (!handshakeResult.success) {
      error(`Handshake with ${relay.toString()} led to empty stream. Giving up.`)
      // Only close the connection to the relay if it does not perform relay services
      // for us.
      if (this.usedRelays.findIndex((usedRelay: PeerId) => usedRelay.equals(relay)) < 0) {
        try {
          await baseConnection.conn.close()
        } catch (err) {
          error(`Error while closing unused connection to relay ${relay.toString()}`, err)
        }
      }
      return
    }

    this.connectedToRelays.add(relay.toString())

    const conn = this.upgradeOutbound(relay, destination, handshakeResult.stream, options)

    log(`successfully established relay connection to ${relay.toString()}`)

    return conn
  }

  private upgradeOutbound(
    relay: PeerId,
    destination: PeerId,
    stream: Stream,
    opts?: HoprConnectDialOptions
  ): MultiaddrConnection {
    if (!!this.testingOptions.__noWebRTCUpgrade) {
      return new RelayConnection({
        stream,
        self: this.libp2p.peerId,
        relay,
        counterparty: destination,
        onReconnect: this._onReconnect
      }) as MultiaddrConnection
    } else {
      let channel = this.webRTCUpgrader.upgradeOutbound()

      let newConn = new RelayConnection({
        stream,
        self: this.libp2p.peerId,
        relay,
        counterparty: destination,
        onReconnect: this._onReconnect,
        webRTC: {
          channel,
          upgradeInbound: this.webRTCUpgrader.upgradeInbound.bind(this.webRTCUpgrader)
        }
      })

      return new WebRTCConnection(newConn, channel, {
        __noWebRTCUpgrade: this.testingOptions.__noWebRTCUpgrade,
        ...opts
      }) as MultiaddrConnection
    }
  }

  private upgradeInbound(initiator: PeerId, relay: PeerId, stream: Stream) {
    if (!!this.testingOptions.__noWebRTCUpgrade) {
      return new RelayConnection({
        stream,
        self: this.libp2p.peerId,
        relay,
        counterparty: initiator,
        onReconnect: this._onReconnect
      })
    } else {
      let channel = this.webRTCUpgrader.upgradeInbound()

      let newConn = new RelayConnection({
        stream,
        self: this.libp2p.peerId,
        relay,
        counterparty: initiator,
        onReconnect: this._onReconnect,
        webRTC: {
          channel,
          upgradeInbound: this.webRTCUpgrader.upgradeInbound.bind(this.webRTCUpgrader)
        }
      })

      return new WebRTCConnection(newConn, channel, {
        __noWebRTCUpgrade: this.testingOptions.__noWebRTCUpgrade
      })
    }
  }

  private async onCanRelay(conn: HandlerProps) {
    // Only called if protocol is supported which
    // means that environments match
    try {
      await conn.stream.sink(
        async function* (this: Relay) {
          // @TODO check if there is a relay slot available

          // Initiate the DHT query but does not await the result which easily
          // takes more than 10 seconds
          ;(async function (this: Relay) {
            try {
              const key = createRelayerKey(conn.connection.remotePeer)

              await this.libp2p.contentRouting.provide(key)

              log(`announced in the DHT as relayer for node ${conn.connection.remotePeer.toString()}`, key)
            } catch (err) {
              error(`error while attempting to provide relayer key for ${conn.connection.remotePeer.toString()}`)
            }
          }.call(this))

          yield OK
        }.call(this)
      )
    } catch (err) {
      error(`Error in CAN_RELAY protocol`, err)
    }
  }

  private async onRelay(conn: HandlerProps) {
    if (conn.connection == undefined || conn.connection.remotePeer == undefined) {
      verbose(`Received incomplete connection object`)
      return
    }

    const shaker = new RelayHandshake(conn.stream as Stream, this.options)

    log(`handling relay request from ${conn.connection.remotePeer.toString()}`)
    log(`relayed connection count: ${this.relayState.relayedConnectionCount()}`)

    try {
      if (this.relayState.relayedConnectionCount() >= (this.options.maxRelayedConnections as number)) {
        log(`relayed request rejected, already at max capacity (${this.options.maxRelayedConnections as number})`)
        await shaker.reject(RelayHandshakeMessage.FAIL_RELAY_FULL)
      } else {
        // NOTE: This cannot be awaited, otherwise it stalls the relay loop. Therefore, promise rejections must
        // be handled downstream to avoid unhandled promise rejection crashes
        shaker.negotiate(
          conn.connection.remotePeer,
          this._dialNodeDirectly as Relay['dialNodeDirectly'],
          this.relayState
        )
      }
    } catch (e) {
      error(`Error while processing relay request from ${conn.connection.remotePeer.toString()}: ${e}`)
    }
  }

  /**
   * Handles incoming relay requests.
   * @param conn incoming connection
   */
  private async onDelivery(conn: HandlerProps): Promise<void> {
    if (conn.stream == undefined || conn.connection == undefined) {
      error(
        `Dropping stream because ${conn.connection == undefined ? 'cannot determine relay address ' : ''}${
          conn.stream == undefined ? 'no stream was given' : ''
        }`
      )
      return
    }

    const handShakeResult = await new RelayHandshake(conn.stream as Stream, this.options).handle(
      conn.connection.remotePeer
    )

    if (!handShakeResult.success) {
      return
    }

    log(`incoming connection from ${handShakeResult.counterparty.toString()}`)

    const newConn = this.upgradeInbound(
      handShakeResult.counterparty,
      conn.connection.remotePeer,
      handShakeResult.stream
    )

    try {
      // Will call internal libp2p event handler, so no further action required
      await this.libp2p.upgrader.upgradeInbound(newConn as MultiaddrConnection)
    } catch (err) {
      error(`Could not upgrade relayed connection. Error was: ${err}`)
      return
    }

    // @TODO
    // this.discovery._peerDiscovered(newConn.remotePeer, [newConn.remoteAddr])
  }

  /**
   * Called once reconnect happens
   * @param newStream new relayed connection
   * @param counterparty counterparty of the relayed connection
   */
  private async onReconnect(newStream: RelayConnection, counterparty: PeerId): Promise<void> {
    log(`####### inside reconnect #######`)

    let newConn: Connection

    log(`Handling reconnection to ${counterparty.toString()}`)

    try {
      if (!!this.testingOptions.__noWebRTCUpgrade) {
        newConn = await this.libp2p.upgrader.upgradeInbound(newStream as MultiaddrConnection)
      } else {
        newConn = await this.libp2p.upgrader.upgradeInbound(
          new WebRTCConnection(newStream, newStream.webRTC!.channel, {
            __noWebRTCUpgrade: this.testingOptions.__noWebRTCUpgrade
          }) as MultiaddrConnection
        )
      }
    } catch (err) {
      error(err)
      return
    }

    // @TODO remove this (1/2 done)
    this.libp2p.dialer._pendingDials?.get(counterparty.toString())?.destroy()

    const existingConnections = this.libp2p.connectionManager.getAll(counterparty)
    for (const existingConnection of existingConnections) {
      if (existingConnection.id === newConn.id) {
        continue
      }
      this.libp2p.connectionManager.onDisconnect(existingConnection)
    }
  }

  /**
   * Attempts to establish a direct connection to the destination
   * @param destination peer to connect to
   * @param protocol
   * @param opts
   * @returns a stream to the given peer
   */
  private async dialNodeDirectly(
    destination: PeerId,
    protocol: string,
    opts?: HoprConnectDialOptions
  ): Promise<ConnResult | void> {
    let connResult = await tryExistingConnections(this.libp2p, destination, protocol)

    // Only establish a new connection if we don't have any.
    // Don't establish a new direct connection to the recipient when using
    // simulated NAT
    if (connResult == undefined) {
      connResult = await this.establishDirectConnection(destination, protocol, opts)
    }

    return connResult
  }

  /**
   * Establishes a new connection to the given by using a direct
   * TCP connection.
   * @param destination peer to connect to
   * @param protocol desired protocol
   * @param opts additional options such as timeout
   * @returns a stream to the given peer
   */
  private async establishDirectConnection(
    destination: PeerId,
    protocol: string,
    opts?: HoprConnectDialOptions
  ): Promise<ConnResult | undefined> {
    const usableAddresses: Multiaddr[] = []

    const knownAddresses: Address[] = await this.libp2p.peerStore.addressBook.get(destination)
    for (const knownAddress of knownAddresses) {
      // Check that the address:
      // - matches the format (PeerStore might include addresses of other transport modules)
      // - is a direct address (PeerStore might include relay addresses)
      if (this.filter([knownAddress.multiaddr]).length > 0 && knownAddress.multiaddr.tuples()[0][0] != CODE_P2P) {
        usableAddresses.push(knownAddress.multiaddr)
      }
    }

    if (usableAddresses.length == 0) {
      return
    }

    let stream: ProtocolStream['stream'] | undefined
    let conn: Connection | undefined

    for (const usable of usableAddresses) {
      try {
        conn = await this.dialDirectly(usable, opts)
      } catch (err) {
        await attemptClose(conn, error)
        continue
      }

      if (conn != undefined) {
        try {
          stream = (await conn.newStream([protocol]))?.stream as ProtocolStream['stream']
        } catch (err) {
          await attemptClose(conn, error)
          continue
        }

        if (
          stream == undefined &&
          // Only close the connection if we are not using this peer as a relay
          this.usedRelays.findIndex((usedRelay: PeerId) => usedRelay.equals(destination)) < 0
        ) {
          await attemptClose(conn, error)
        }
      }
    }
    return conn != undefined && stream != undefined ? { conn, stream, protocol } : undefined
  }
}

export { Relay }
