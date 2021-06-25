/// <reference path="../@types/it-handshake.ts" />
/// <reference path="../@types/libp2p.ts" />
/// <reference path="../@types/libp2p-interfaces.ts" />

import debug from 'debug'

const DEBUG_PREFIX = 'hopr-connect:relay'

const log = debug(DEBUG_PREFIX.concat(':relay'))
const error = debug(DEBUG_PREFIX.concat(':error'))
const verbose = debug(DEBUG_PREFIX.concat(':verbose'))

import libp2p from 'libp2p'
import { WebRTCUpgrader } from '../webrtc'

import PeerId from 'peer-id'

import { RELAY_CIRCUIT_TIMEOUT, RELAY, DELIVERY } from '../constants'

import { RelayConnection } from './connection'
import { WebRTCConnection } from '../webRTCConnection'

import type { Connection } from 'libp2p-interfaces'
import type { DialOptions, Handler, Stream, ConnHandler, Dialer, ConnectionManager, Upgrader } from 'libp2p'
import { AbortError } from 'abortable-iterator'
import { dialHelper } from '../utils'
import { RelayHandshake } from './handshake'
import { RelayState } from './state'

class Relay {
  private _connectionManager: ConnectionManager
  private _dialer: Dialer
  private relayState: RelayState

  constructor(
    private libp2p: libp2p,
    private upgrader: Upgrader,
    private connHandler: ConnHandler | undefined,
    private webRTCUpgrader?: WebRTCUpgrader,
    private __noWebRTCUpgrade?: boolean
  ) {
    this._connectionManager = libp2p.connectionManager
    this._dialer = libp2p.dialer

    this.relayState = new RelayState()

    libp2p.handle(DELIVERY, this.handleIncoming.bind(this))

    libp2p.handle(RELAY, ({ stream, connection }) => {
      if (connection == undefined || connection.remotePeer == undefined) {
        verbose(`Received incomplete connection object`)
        return
      }

      new RelayHandshake(stream).negotiate(
        connection.remotePeer,
        (counterparty: PeerId) => this.contactCounterparty(counterparty),
        this.relayState.exists.bind(this.relayState),
        this.relayState.isActive.bind(this.relayState),
        this.relayState.updateExisting.bind(this.relayState),
        this.relayState.createNew.bind(this.relayState)
      )
    })
  }

  async connect(
    relay: PeerId,
    destination: PeerId,
    options?: DialOptions
  ): Promise<RelayConnection | WebRTCConnection | undefined> {
    const opts =
      options != undefined && options.signal != undefined
        ? { signal: options.signal }
        : { timeout: RELAY_CIRCUIT_TIMEOUT }

    const relayConnection = await dialHelper(this.libp2p, relay, RELAY, opts)

    if (relayConnection == undefined) {
      error(`Could not establish relayed conntection over ${relay.toB58String()} to ${destination.toB58String()}`)
      return
    }

    const handshakeResult = await new RelayHandshake(relayConnection.stream).initiate(relay, destination)

    if (!handshakeResult.success) {
      error(`Handshake led to empty stream. Giving up.`)
      return
    }

    if (options?.signal?.aborted) {
      throw new AbortError()
    }

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

      return new WebRTCConnection(
        {
          conn: newConn,
          self: this.libp2p.peerId,
          counterparty: destination,
          channel,
          libp2p: {
            connectionManager: this._connectionManager
          } as any
        },
        {
          __noWebRTCUpgrade: this.__noWebRTCUpgrade,
          ...options
        }
      )
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

  async handleRelayConnection(
    conn: Handler,
    onReconnect: (newStream: RelayConnection, counterparty: PeerId) => Promise<void>
  ): Promise<RelayConnection | WebRTCConnection | undefined> {
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
        self: this.libp2p.peerId,
        relay: conn.connection.remotePeer,
        counterparty: handShakeResult.counterparty,
        onReconnect,
        webRTC: {
          channel,
          upgradeInbound: this.webRTCUpgrader.upgradeInbound.bind(this.webRTCUpgrader)
        }
      })

      return new WebRTCConnection(
        {
          conn: newConn,
          self: this.libp2p.peerId,
          counterparty: handShakeResult.counterparty,
          channel,
          libp2p: {
            connectionManager: this._connectionManager
          } as any
        },
        { __noWebRTCUpgrade: this.__noWebRTCUpgrade }
      )
    } else {
      return new RelayConnection({
        stream: handShakeResult.stream,
        self: this.libp2p.peerId,
        relay: conn.connection.remotePeer,
        counterparty: handShakeResult.counterparty,
        onReconnect
      })
    }
  }

  /**
   * Called once a relayed connection is establishing
   * @param handler handles the relayed connection
   */
  private async handleIncoming(handler: Handler): Promise<void> {
    let newConn: Connection

    try {
      const relayConnection = await this.handleRelayConnection(handler, this.onReconnect.bind(this))

      if (relayConnection == undefined) {
        return
      }

      newConn = await this.upgrader.upgradeInbound(relayConnection)
    } catch (err) {
      error(`Could not upgrade relayed connection. Error was: ${err}`)
      return
    }

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
          new WebRTCConnection(
            {
              conn: newStream,
              self: this.libp2p.peerId,
              counterparty,
              channel: newStream.webRTC!.channel,
              libp2p: {
                connectionManager: this._connectionManager
              } as any
            },
            {
              __noWebRTCUpgrade: this.__noWebRTCUpgrade
            }
          )
        )
      } else {
        newConn = await this.upgrader.upgradeInbound(newStream)
      }
    } catch (err) {
      error(err)
      return
    }

    this._dialer._pendingDials[counterparty.toB58String()]?.destroy()
    this._connectionManager.connections.set(counterparty.toB58String(), [newConn])

    this.connHandler?.(newConn)
  }

  private async contactCounterparty(counterparty: PeerId): Promise<Stream | undefined> {
    // @TODO this produces struct with unset connection property
    let newConn = await dialHelper(this.libp2p, counterparty, DELIVERY, { timeout: RELAY_CIRCUIT_TIMEOUT })

    if (newConn != undefined && newConn.connection == undefined) {
      verbose(`DEBUG: Received incomplete connection object. Connection object:`, newConn)
      return undefined
    }

    return newConn?.stream
  }
}

export { Relay }
