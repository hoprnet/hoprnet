import type { HandlerProps } from 'libp2p'
import type { Connection } from 'libp2p-interfaces/connection'
import type { ConnectionHandler } from 'libp2p-interfaces/transport'
import type { Stream, HoprConnectDialOptions } from '../types'
import type Libp2p from 'libp2p'
import type PeerId from 'peer-id'
import { AbortError } from 'abortable-iterator'

import debug from 'debug'

import { WebRTCUpgrader, WebRTCConnection } from '../webrtc'
import chalk from 'chalk'
import { RELAY_CIRCUIT_TIMEOUT, RELAY, DELIVERY } from '../constants'
import { RelayConnection } from './connection'
import { RelayHandshake, RelayHandshakeMessage } from './handshake'
import { RelayState } from './state'

import type HoprConnect from '..'

import { Multiaddr } from 'multiaddr'

const DEBUG_PREFIX = 'hopr-connect:relay'
const DEFAULT_MAX_RELAYED_CONNECTIONS = 10

const log = debug(DEBUG_PREFIX)
const error = debug(DEBUG_PREFIX.concat(':error'))
const verbose = debug(DEBUG_PREFIX.concat(':verbose'))

// Specify which libp2p methods this class uses
// such that Typescript fails to build if anything changes
type ReducedPeerStore = {
  peerStore: {
    get: (peer: PeerId) => Pick<NonNullable<ReturnType<Libp2p['peerStore']['get']>>, 'addresses'> | undefined
  }
}
type ReducedDialer = { dialer: Pick<Libp2p['dialer'], '_pendingDials'> }
type ReducedUpgrader = { upgrader: Pick<Libp2p['upgrader'], 'upgradeInbound' | 'upgradeOutbound'> }
type ReducedConnectionManager = { connectionManager: Pick<Libp2p['connectionManager'], 'connections' | 'get'> }
type ReducedDHT = { peerRouting: Pick<Libp2p['peerRouting'], '_routers' | 'findPeer'> }
type ReducedLibp2p = ReducedPeerStore &
  ReducedDialer &
  ReducedDHT &
  ReducedConnectionManager &
  ReducedUpgrader & {
    peerId: PeerId
    handle: Libp2p['handle']
    dialProtocol: Libp2p['dialProtocol']
  }

/**
 * API interface for relayed connections
 */
class Relay {
  private relayState: RelayState

  constructor(
    public libp2p: ReducedLibp2p,
    private dialDirectly: HoprConnect['dialDirectly'],
    private filter: HoprConnect['filter'],
    private connHandler: ConnectionHandler | undefined,
    private webRTCUpgrader?: WebRTCUpgrader,
    private __noWebRTCUpgrade?: boolean,
    private maxRelayedConnections: number = DEFAULT_MAX_RELAYED_CONNECTIONS,
    private __relayFreeTimeout?: number
  ) {
    this.relayState = new RelayState()
  }

  /**
   * Assigns the event listener to the constructed object.
   * @dev Must not happen in the constructor because `this` is not ready
   *      at that point in time
   */
  start(): void {
    this.libp2p.handle(DELIVERY, this.handleIncoming.bind(this))

    this.libp2p.handle(RELAY, ({ stream, connection }) => {
      if (connection == undefined || connection.remotePeer == undefined) {
        verbose(`Received incomplete connection object`)
        return
      }

      const shaker = new RelayHandshake(stream as any)

      log(`handling relay request from ${connection.remotePeer.toB58String()}`)
      log(`relayed connection count: ${this.relayState.relayedConnectionCount()}`)

      if (this.relayState.relayedConnectionCount() >= this.maxRelayedConnections) {
        log(`relayed request rejected, already at max capacity (${this.maxRelayedConnections})`)
        shaker.reject(RelayHandshakeMessage.FAIL_RELAY_FULL)
      } else {
        shaker.negotiate(
          connection.remotePeer,
          this.dialNodeDirectly.bind(this),
          this.relayState.exists.bind(this.relayState),
          this.relayState.isActive.bind(this.relayState),
          this.relayState.updateExisting.bind(this.relayState),
          this.relayState.createNew.bind(this.relayState),
          this.__relayFreeTimeout
        )
      }
    })
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
  ): Promise<RelayConnection | WebRTCConnection | undefined> {
    const baseConnection = await this.dialNodeDirectly(relay, RELAY, {
      timeout: RELAY_CIRCUIT_TIMEOUT,
      signal: options?.signal
    })

    if (baseConnection == undefined) {
      error(
        `Cannot establish a connection to ${chalk.green(destination.toB58String())} because relay ${chalk.green(
          relay.toB58String()
        )} is not reachable`
      )
      return
    }

    const handshakeResult = await new RelayHandshake(baseConnection).initiate(relay, destination)

    if (!handshakeResult.success) {
      error(`Handshake led to empty stream. Giving up.`)
      return
    }

    if (options?.signal?.aborted) {
      throw new AbortError()
    }

    return this.upgradeOutbound(relay, destination, handshakeResult.stream, options)
  }

  private upgradeOutbound(relay: PeerId, destination: PeerId, stream: Stream, opts?: HoprConnectDialOptions) {
    // Attempt to upgrade to WebRTC if available
    if (this.webRTCUpgrader != undefined) {
      let channel = this.webRTCUpgrader.upgradeOutbound()

      let newConn = new RelayConnection({
        stream,
        self: this.libp2p.peerId,
        relay,
        counterparty: destination,
        onReconnect: this.onReconnect.bind(this),
        webRTC: {
          channel,
          upgradeInbound: this.webRTCUpgrader.upgradeInbound.bind(this.webRTCUpgrader)
        }
      })

      return new WebRTCConnection(destination, this.libp2p.connectionManager, newConn, channel, {
        __noWebRTCUpgrade: this.__noWebRTCUpgrade,
        ...opts
      } as any)
    } else {
      return new RelayConnection({
        stream,
        self: this.libp2p.peerId,
        relay,
        counterparty: destination,
        onReconnect: this.onReconnect.bind(this)
      })
    }
  }

  /**
   * Handle an incoming relayed connection and perfom handshake
   * @param conn base connection to use
   * @returns the relayed connection if the handshake were successful
   */
  private async handleRelayConnection(conn: HandlerProps): Promise<RelayConnection | WebRTCConnection | undefined> {
    if (conn.stream == undefined || conn.connection == undefined) {
      error(
        `Dropping stream because ${conn.connection == undefined ? 'cannot determine relay address ' : ''}${
          conn.stream == undefined ? 'no stream was given' : ''
        }`
      )
      return
    }

    const handShakeResult = await new RelayHandshake(conn.stream as any).handle(conn.connection.remotePeer)

    if (!handShakeResult.success) {
      return
    }

    log(`incoming connection from ${handShakeResult.counterparty.toB58String()}`)

    return this.upgradeInbound(handShakeResult.counterparty, conn.connection.remotePeer, handShakeResult.stream)
  }

  private upgradeInbound(initiator: PeerId, relay: PeerId, stream: Stream) {
    if (this.webRTCUpgrader != undefined) {
      let channel = this.webRTCUpgrader.upgradeInbound()

      let newConn = new RelayConnection({
        stream,
        self: this.libp2p.peerId,
        relay,
        counterparty: initiator,
        onReconnect: this.onReconnect.bind(this),
        webRTC: {
          channel,
          upgradeInbound: this.webRTCUpgrader.upgradeInbound.bind(this.webRTCUpgrader)
        }
      })

      return new WebRTCConnection(initiator, this.libp2p.connectionManager, newConn, channel, {
        __noWebRTCUpgrade: this.__noWebRTCUpgrade
      } as any)
    } else {
      return new RelayConnection({
        stream,
        self: this.libp2p.peerId,
        relay,
        counterparty: initiator,
        onReconnect: this.onReconnect.bind(this)
      })
    }
  }

  /**
   * Called once a relayed connection is about to get established.
   * Forwards the connection to libp2p if no errors happened
   * @param handler handles the relayed connection
   */
  private async handleIncoming(handler: HandlerProps): Promise<void> {
    let newConn: Connection

    try {
      const relayConnection = await this.handleRelayConnection(handler)

      if (relayConnection == undefined) {
        return
      }

      newConn = (await this.libp2p.upgrader.upgradeInbound(relayConnection as any)) as any
    } catch (err) {
      error(`Could not upgrade relayed connection. Error was: ${err}`)
      return
    }

    // @TODO
    // this.discovery._peerDiscovered(newConn.remotePeer, [newConn.remoteAddr])

    this.connHandler?.(newConn)
  }

  /**
   * Dialed once a reconnect happens
   * @param newStream new relayed connection
   * @param counterparty counterparty of the relayed connection
   */
  private async onReconnect(newStream: RelayConnection, counterparty: PeerId): Promise<void> {
    log(`####### inside reconnect #######`)

    let newConn: Connection

    try {
      if (this.webRTCUpgrader != undefined) {
        newConn = (await this.libp2p.upgrader.upgradeInbound(
          new WebRTCConnection(counterparty, this.libp2p.connectionManager, newStream, newStream.webRTC!.channel, {
            __noWebRTCUpgrade: this.__noWebRTCUpgrade
          }) as any
        )) as any
      } else {
        newConn = (await this.libp2p.upgrader.upgradeInbound(newStream as any)) as any
      }
    } catch (err) {
      error(err)
      return
    }

    // @TODO remove this
    this.libp2p.dialer._pendingDials?.get(counterparty.toB58String())?.destroy()
    this.libp2p.connectionManager.connections?.set(counterparty.toB58String(), [newConn as any])

    this.connHandler?.(newConn)
  }

  /**
   * Attempts to establish a direct connection to counterparty
   * @param counterparty peerId of counterparty
   * @returns a duplex stream to counterparty, if connection is possible
   */
  private async dialNodeDirectly(
    counterparty: PeerId,
    protocol: string,
    opts?: HoprConnectDialOptions
  ): Promise<Stream | undefined> {
    let stream = await this.tryExistingConnection(counterparty, protocol)

    if (stream == undefined) {
      stream = await this.establishDirectConnection(counterparty, protocol, opts)
    }

    return stream
  }

  private async establishDirectConnection(
    peer: PeerId,
    protocol: string,
    opts?: HoprConnectDialOptions
  ): Promise<Stream | undefined> {
    const usableAddresses: Multiaddr[] = []

    for (const knownAddress of this.libp2p.peerStore.get(peer)?.addresses ?? []) {
      if (this.filter([knownAddress.multiaddr])) {
        usableAddresses.push(knownAddress.multiaddr)
      }
    }

    if (usableAddresses.length == 0) {
      return
    }

    let stream: Stream | undefined
    for (const usable of usableAddresses) {
      let conn: Connection
      try {
        conn = await this.dialDirectly(usable, opts)
      } catch (err) {
        continue
      }

      if (conn != undefined) {
        try {
          stream = (await conn.newStream([protocol])) as any
        } catch (err) {
          console.log(`stream failed`, usable.toString(), err)

          continue
        }
      }
    }
    console.log(`stream successful`)

    return stream
  }

  private async tryExistingConnection(peer: PeerId, protocol: string): Promise<Stream | undefined> {
    const existingConnection = this.libp2p.connectionManager.get(peer)
    if (existingConnection == null) {
      return
    }

    let stream: Stream
    try {
      stream = (await existingConnection.newStream(protocol)).stream as any
    } catch (err) {
      return
    }

    return stream
  }
}

export { Relay }
