import type { HandlerProps, Connection } from 'libp2p'
import type { Stream, DialOptions } from '../types'
import type Libp2p from 'libp2p'
import type PeerId from 'peer-id'
import { AbortError } from 'abortable-iterator'

import { dial as dialHelper } from '@hoprnet/hopr-utils'

import debug from 'debug'

import { WebRTCUpgrader, WebRTCConnection } from '../webrtc'
import { green } from 'chalk'
import { RELAY_CIRCUIT_TIMEOUT, RELAY, DELIVERY } from '../constants'
import { RelayConnection } from './connection'
import { RelayHandshake, RelayHandshakeMessage } from './handshake'
import { RelayState } from './state'

const DEBUG_PREFIX = 'hopr-connect:relay'
const DEFAULT_MAX_RELAYED_CONNECTIONS = 10

const log = debug(DEBUG_PREFIX)
const error = debug(DEBUG_PREFIX.concat(':error'))
const verbose = debug(DEBUG_PREFIX.concat(':verbose'))

/**
 * API interface for relayed connections
 */
class Relay {
  private relayState: RelayState

  constructor(
    public libp2p: Libp2p,
    private connHandler: ((conn: Connection) => void) | undefined,
    private webRTCUpgrader?: WebRTCUpgrader,
    private __noWebRTCUpgrade?: boolean,
    private maxRelayedConnections: number = DEFAULT_MAX_RELAYED_CONNECTIONS,
    private __relayFreeTimeout?: number
  ) {
    this.relayState = new RelayState()

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
          (counterparty: PeerId) => this.contactCounterparty(counterparty),
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
    options?: DialOptions
  ): Promise<RelayConnection | WebRTCConnection | undefined> {

    options?.signal
    const baseConnection = await dialHelper(this.libp2p, relay, RELAY, {
      timeout: RELAY_CIRCUIT_TIMEOUT,
      signal: options?.signal
    })

    if (baseConnection.status !== 'SUCCESS') {
      error(
        `Could not establish relayed conntection over ${green(relay.toB58String())} to ${green(
          destination.toB58String()
        )}`, baseConnection.status
      )
      return
    }

    const handshakeResult = await new RelayHandshake(baseConnection.resp.stream as any).initiate(relay, destination)

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
        ...options
      } as any)
    } else {
      return new RelayConnection({
        stream: handshakeResult.stream,
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

    log(`counterparty relayed connection established`)

    if (this.webRTCUpgrader != undefined) {
      let channel = this.webRTCUpgrader.upgradeInbound()

      let newConn = new RelayConnection({
        stream: handShakeResult.stream,
        self: this.libp2p.peerId,
        relay: conn.connection.remotePeer,
        counterparty: handShakeResult.counterparty,
        onReconnect: this.onReconnect.bind(this),
        webRTC: {
          channel,
          upgradeInbound: this.webRTCUpgrader.upgradeInbound.bind(this.webRTCUpgrader)
        }
      })

      return new WebRTCConnection(handShakeResult.counterparty, this.libp2p.connectionManager, newConn, channel, {
        __noWebRTCUpgrade: this.__noWebRTCUpgrade
      } as any)
    } else {
      return new RelayConnection({
        stream: handShakeResult.stream,
        self: this.libp2p.peerId,
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
  private async handleIncoming(handler: HandlerProps): Promise<void> {
    let newConn: Connection

    try {
      const relayConnection = await this.handleRelayConnection(handler)

      if (relayConnection == undefined) {
        return
      }

      newConn = await this.libp2p.upgrader.upgradeInbound(relayConnection as any)
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
        newConn = await this.libp2p.upgrader.upgradeInbound(
          new WebRTCConnection(counterparty, this.libp2p.connectionManager, newStream, newStream.webRTC!.channel, {
            __noWebRTCUpgrade: this.__noWebRTCUpgrade
          }) as any
        )
      } else {
        newConn = await this.libp2p.upgrader.upgradeInbound(newStream as any)
      }
    } catch (err) {
      error(err)
      return
    }

    // @TODO remove this
    this.libp2p.dialer._pendingDials?.get(counterparty.toB58String())?.destroy()
    this.libp2p.connectionManager.connections?.set(counterparty.toB58String(), [newConn])

    this.connHandler?.(newConn)
  }

  /**
   * Attempts to establish a direct connection to counterparty
   * @param counterparty peerId of counterparty
   * @returns a duplex stream to counterparty, if connection is possible
   */
  private async contactCounterparty(counterparty: PeerId): Promise<Stream | undefined> {
    // @TODO this produces struct with unset connection property
    let newConn = await dialHelper(this.libp2p, counterparty, DELIVERY, { timeout: RELAY_CIRCUIT_TIMEOUT })

    if (newConn.status === 'SUCCESS') {
      return newConn.resp.stream as any
    }

    return undefined
  }
}

export { Relay }
