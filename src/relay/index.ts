/// <reference path="../@types/it-handshake.ts" />
/// <reference path="../@types/libp2p.ts" />
/// <reference path="../@types/libp2p-interfaces.ts" />

import debug from 'debug'

const DEBUG_PREFIX = 'hopr-connect:relay'
const DEFAULT_MAX_RELAYED_CONNECTIONS = 10

const log = debug(DEBUG_PREFIX)
const error = debug(DEBUG_PREFIX.concat(':error'))
const verbose = debug(DEBUG_PREFIX.concat(':verbose'))

import { WebRTCUpgrader, WebRTCConnection } from '../webrtc'

import PeerId from 'peer-id'
import { green } from 'chalk'

import { RELAY_CIRCUIT_TIMEOUT, RELAY, DELIVERY } from '../constants'

import { RelayConnection } from './connection'

import type { Connection } from 'libp2p-interfaces'
import type { DialOptions, Handler, Stream, ConnHandler, Dialer, ConnectionManager, Upgrader } from 'libp2p'
import { AbortError } from 'abortable-iterator'
import { RelayHandshake, RelayHandshakeMessage } from './handshake'
import { RelayState } from './state'

/**
 * API interface for relayed connections
 */
class Relay {
  private relayState: RelayState

  constructor(
    private dialHelper: (
      peer: PeerId,
      protocol: string,
      opts: { timeout: number } | { signal: AbortSignal }
    ) => Promise<Handler | undefined>,
    private dialer: Dialer,
    private connectionManager: ConnectionManager,
    private handle: (protocol: string, handle: (handler: Handler) => void) => void,
    public peerId: PeerId,
    private upgrader: Upgrader,
    private connHandler: ConnHandler | undefined,
    private webRTCUpgrader?: WebRTCUpgrader,
    private __noWebRTCUpgrade?: boolean,
    private maxRelayedConnections: number = DEFAULT_MAX_RELAYED_CONNECTIONS
  ) {
    this.relayState = new RelayState()

    this.handle(DELIVERY, this.handleIncoming.bind(this))

    this.handle(RELAY, ({ stream, connection }) => {
      if (connection == undefined || connection.remotePeer == undefined) {
        verbose(`Received incomplete connection object`)
        return
      }

      const shaker = new RelayHandshake(stream)

      log(`handling relay request from ${connection.remotePeer}`)
      log(`relayed connection count: ${this.relayState.relayedConnectionCount()}`)

      if (this.relayState.relayedConnectionCount() >= this.maxRelayedConnections) {
        log(`relayed request rejected, already at max capacity (${this.maxRelayedConnections})`)
        shaker.reject(RelayHandshakeMessage.FAIL_RELAY_FULL)
      } else {
        shaker.negotiate(
          connection.remotePeer,
          (counterparty: PeerId) => this.contactCounterparty(counterparty),
          this.relayState.exists.bind(this.relayState),
          this.relayState.isActive.bind(this.relayState),
          this.relayState.updateExisting.bind(this.relayState),
          this.relayState.createNew.bind(this.relayState)
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
    options?: DialOptions
  ): Promise<RelayConnection | WebRTCConnection | undefined> {
    const opts =
      options != undefined && options.signal != undefined
        ? { signal: options.signal }
        : { timeout: RELAY_CIRCUIT_TIMEOUT }

    const baseConnection = await this.dialHelper(relay, RELAY, opts)

    if (baseConnection == undefined) {
      error(
        `Could not establish relayed conntection over ${green(relay.toB58String())} to ${green(
          destination.toB58String()
        )}`
      )
      return
    }

    const handshakeResult = await new RelayHandshake(baseConnection.stream).initiate(relay, destination)

    if (!handshakeResult.success) {
      error(`Handshake led to empty stream. Giving up.`)
      return
    }

    if (options?.signal?.aborted) {
      throw new AbortError()
    }

    // Attempt to upgrade to WebRTC if available
    if (this.webRTCUpgrader != undefined) {
      let channel = this.webRTCUpgrader.upgradeOutbound()

      let newConn = new RelayConnection({
        stream: handshakeResult.stream,
        self: this.peerId,
        relay,
        counterparty: destination,
        onReconnect: this.onReconnect.bind(this),
        webRTC: {
          channel,
          upgradeInbound: this.webRTCUpgrader.upgradeInbound.bind(this.webRTCUpgrader)
        }
      })

      return new WebRTCConnection(destination, this.connectionManager, newConn, channel, {
        __noWebRTCUpgrade: this.__noWebRTCUpgrade,
        ...options
      })
    } else {
      return new RelayConnection({
        stream: handshakeResult.stream,
        self: this.peerId,
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
  private async handleRelayConnection(conn: Handler): Promise<RelayConnection | WebRTCConnection | undefined> {
    if (conn.stream == undefined || conn.connection == undefined) {
      error(
        `Dropping stream because ${conn.connection == undefined ? 'cannot determine relay address ' : ''}${
          conn.stream == undefined ? 'no stream was given' : ''
        }`
      )
      return
    }

    const handShakeResult = await new RelayHandshake(conn.stream).handle(conn.connection.remotePeer)

    if (!handShakeResult.success) {
      return
    }

    log(`incoming connection from ${handShakeResult.counterparty.toB58String()}`)

    log(`counterparty relayed connection established`)

    if (this.webRTCUpgrader != undefined) {
      let channel = this.webRTCUpgrader.upgradeInbound()

      let newConn = new RelayConnection({
        stream: handShakeResult.stream,
        self: this.peerId,
        relay: conn.connection.remotePeer,
        counterparty: handShakeResult.counterparty,
        onReconnect: this.onReconnect.bind(this),
        webRTC: {
          channel,
          upgradeInbound: this.webRTCUpgrader.upgradeInbound.bind(this.webRTCUpgrader)
        }
      })

      return new WebRTCConnection(handShakeResult.counterparty, this.connectionManager, newConn, channel, {
        __noWebRTCUpgrade: this.__noWebRTCUpgrade
      })
    } else {
      return new RelayConnection({
        stream: handShakeResult.stream,
        self: this.peerId,
        relay: conn.connection.remotePeer,
        counterparty: handShakeResult.counterparty,
        onReconnect: this.onReconnect.bind(this)
      })
    }
  }

  /**
   * Called once a relayed connection is about to get established.
   * Forwards the connection to libp2p if no errors happened
   * @param handler handles the relayed connection
   */
  private async handleIncoming(handler: Handler): Promise<void> {
    let newConn: Connection

    try {
      const relayConnection = await this.handleRelayConnection(handler)

      if (relayConnection == undefined) {
        return
      }

      newConn = await this.upgrader.upgradeInbound(relayConnection)
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
        newConn = await this.upgrader.upgradeInbound(
          new WebRTCConnection(counterparty, this.connectionManager, newStream, newStream.webRTC!.channel, {
            __noWebRTCUpgrade: this.__noWebRTCUpgrade
          })
        )
      } else {
        newConn = await this.upgrader.upgradeInbound(newStream)
      }
    } catch (err) {
      error(err)
      return
    }

    // @TODO remove this
    this.dialer._pendingDials?.[counterparty.toB58String()]?.destroy()
    this.connectionManager.connections?.set(counterparty.toB58String(), [newConn])

    this.connHandler?.(newConn)
  }

  /**
   * Attempts to establish a direct connection to counterparty
   * @param counterparty peerId of counterparty
   * @returns a duplex stream to counterparty, if connection is possible
   */
  private async contactCounterparty(counterparty: PeerId): Promise<Stream | undefined> {
    // @TODO this produces struct with unset connection property
    let newConn = await this.dialHelper(counterparty, DELIVERY, { timeout: RELAY_CIRCUIT_TIMEOUT })

    // @TODO
    // if (newConn != undefined && newConn.connection == undefined) {
    //   verbose(`DEBUG: Received incomplete connection object. Connection object:`, newConn)
    //   return undefined
    // }

    return newConn?.stream
  }
}

export { Relay }
