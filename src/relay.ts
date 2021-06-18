/// <reference path="./@types/it-handshake.ts" />
/// <reference path="./@types/libp2p.ts" />
/// <reference path="./@types/libp2p-interfaces.ts" />

import debug from 'debug'
const log = debug('hopr-connect')
const error = debug('hopr-connect:error')
const verbose = debug('hopr-connect:verbose:error')

import { blue, yellow } from 'chalk'
import libp2p from 'libp2p'
import { WebRTCUpgrader } from './webrtc'

import handshake from 'it-handshake'

import PeerId from 'peer-id'

import {
  RELAY_CIRCUIT_TIMEOUT,
  RELAY,
  DELIVERY,
  OK,
  FAIL,
  FAIL_COULD_NOT_REACH_COUNTERPARTY,
  FAIL_COULD_NOT_IDENTIFY_PEER,
  FAIL_LOOPBACKS_ARE_NOT_ALLOWED,
  FAIL_INVALID_PUBLIC_KEY
} from './constants'

import { u8aCompare, u8aEquals, pubKeyToPeerId } from '@hoprnet/hopr-utils'

import { RelayContext } from './relayContext'

import { RelayConnection } from './relayConnection'
import { WebRTCConnection } from './webRTCConnection'

import type { Connection } from 'libp2p-interfaces'
import type { DialOptions, Handler, Stream, ConnHandler, Dialer, ConnectionManager, Upgrader } from 'libp2p'
import { AbortError } from 'abortable-iterator'
import { dialHelper } from './utils'
import { BLInterface } from 'bl'

class Relay {
  private _streams: Map<string, { [index: string]: RelayContext }>
  private _connectionManager: ConnectionManager
  private _dialer: Dialer

  constructor(
    private libp2p: libp2p,
    private upgrader: Upgrader,
    private connHandler: ConnHandler | undefined,
    private webRTCUpgrader?: WebRTCUpgrader,
    private __noWebRTCUpgrade?: boolean
  ) {
    this._connectionManager = libp2p.connectionManager
    this._dialer = libp2p.dialer

    this._streams = new Map<string, { [index: string]: RelayContext }>()

    libp2p.handle(DELIVERY, this.handleIncoming.bind(this))

    libp2p.handle(RELAY, this.handleRelay.bind(this))
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

    const stream = await this.performHandshake(relayConnection.stream, relay, destination)

    if (stream == null) {
      error(`Handshake led to empty stream. Giving up.`)
      return
    }

    if (options?.signal?.aborted) {
      throw new AbortError()
    }

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
        stream,
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

    const handShakeResult = await this.handleHandshake(conn.stream)

    if (handShakeResult == undefined) {
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

  private async performHandshake(stream: Stream, relay: PeerId, destination: PeerId): Promise<Stream | undefined> {
    let shaker = handshake<Uint8Array | BLInterface>(stream)

    shaker.write(destination.pubKey.marshal())

    let answer: Uint8Array | undefined
    try {
      answer = (await shaker.read())?.slice()
    } catch (err) {
      error(`Error while reading answer ${blue(relay.toB58String())}. ${err.message}`)
      return
    }

    shaker.rest()

    if (answer == null || !u8aEquals(answer, OK)) {
      if (u8aEquals(answer, FAIL_COULD_NOT_REACH_COUNTERPARTY)) {
        await this.libp2p.hangUp(destination)
        return
      }
      error(
        `Could not establish relayed connection to ${blue(
          destination.toB58String()
        )} over relay ${relay.toB58String()}. Answer was: <${yellow(
          new TextDecoder().decode(answer ?? Uint8Array.from([]))
        )}>`
      )
      return
    }

    return shaker.stream
  }

  private async handleHandshake(stream: Stream): Promise<{ stream: Stream; counterparty: PeerId } | undefined> {
    let shaker = handshake<Uint8Array | BLInterface>(stream)

    let pubKeySender: Uint8Array | undefined
    try {
      pubKeySender = (await shaker.read())?.slice()
    } catch (err) {
      error(err)
    }

    if (pubKeySender == undefined || pubKeySender == null) {
      error(`Received empty message. Ignoring connection ...`)
      shaker.write(FAIL)
      shaker.rest()
      return
    }

    let counterparty: PeerId
    try {
      counterparty = pubKeyToPeerId(pubKeySender)
    } catch (err) {
      error(`Could not decode sender peerId. Error was: ${err.message}`)
      shaker.write(FAIL)
      shaker.rest()
      return
    }

    shaker.write(OK)
    shaker.rest()

    return { stream: shaker.stream, counterparty }
  }

  private async handleRelay({ stream, connection }: Handler): Promise<void> {
    log(`handle relay request`)
    const shaker = handshake<Uint8Array | BLInterface>(stream)

    let pubKeySender: Uint8Array | undefined

    if (connection == undefined || connection.remotePeer == undefined) {
      error(`Could not identify peer. Ending relayed connection.`)
      shaker.write(FAIL_COULD_NOT_IDENTIFY_PEER)
      shaker.rest()
      return
    }

    try {
      pubKeySender = (await shaker.read())?.slice()
    } catch (err) {
      error(err)
    }

    if (pubKeySender == null) {
      error(
        `Received empty message from peer ${yellow(
          connection.remotePeer.toB58String()
        )}. Ending stream because we cannot identify counterparty.`
      )
      shaker.write(FAIL)
      shaker.rest()
      return
    }

    let counterparty: PeerId
    try {
      counterparty = pubKeyToPeerId(pubKeySender)
    } catch (err) {
      error(
        `Peer ${yellow(
          connection.remotePeer.toB58String()
        )} asked to establish relayed connection to invalid counterparty. Error was ${err}. Received message ${pubKeySender}`
      )
      shaker.write(FAIL_INVALID_PUBLIC_KEY)
      shaker.rest()
      return
    }

    log(`counterparty identified as ${counterparty.toB58String()}`)

    if (connection.remotePeer != null && counterparty.equals(connection.remotePeer)) {
      error(`Peer ${connection.remotePeer} is trying to loopback to itself. Dropping connection.`)
      shaker.write(FAIL_LOOPBACKS_ARE_NOT_ALLOWED)
      shaker.rest()
      return
    }

    const channelId = getId(connection.remotePeer, counterparty)

    let contextEntry = this._streams.get(channelId)

    if (contextEntry != undefined) {
      verbose(`Relay context between ${connection.remotePeer.toB58String()} and ${counterparty.toB58String()} exists.`)

      const latency = await contextEntry[counterparty.toB58String()].ping()

      verbose(`Latency to ${connection.remotePeer.toB58String()}: ${latency}ms`)
      if (latency >= 0) {
        verbose(`stream to ${counterparty.toB58String()} is alive (latency: ${latency} ms). Using existing stream`)

        shaker.write(OK)
        shaker.rest()

        contextEntry[connection.remotePeer.toB58String()].update(shaker.stream)

        return
      }
      verbose(`stream to ${counterparty.toB58String()} is NOT alive. Establishing a new one`)
    } else {
      log(
        `${connection.remotePeer.toB58String()} to ${counterparty.toB58String()} had no connection. Establishing a new one`
      )
    }

    const deliveryStream = await this.contactCounterparty(connection.remotePeer, counterparty)

    if (deliveryStream == undefined) {
      shaker.write(FAIL_COULD_NOT_REACH_COUNTERPARTY)
      shaker.rest()

      if (contextEntry != undefined) {
        // @TODO close previous instances
        this._streams.delete(channelId)
      }

      return
    }

    shaker.write(OK)
    shaker.rest()

    const senderContext = new RelayContext(shaker.stream)
    const counterpartyContext = new RelayContext(deliveryStream)

    senderContext.sink(counterpartyContext.source)
    counterpartyContext.sink(senderContext.source)

    contextEntry = {
      [connection.remotePeer.toB58String()]: senderContext,
      [counterparty.toB58String()]: counterpartyContext
    }

    this._streams.set(channelId, contextEntry)
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

  private async contactCounterparty(initiator: PeerId, counterparty: PeerId): Promise<Stream | undefined> {
    // @TODO this produces struct with unset connection property
    let newConn = await dialHelper(this.libp2p, counterparty, DELIVERY, { timeout: RELAY_CIRCUIT_TIMEOUT })

    if (newConn != undefined && newConn.connection == undefined) {
      verbose(`DEBUG: Received incomplete connection object. Connection object:`, newConn)
    }

    if (newConn == undefined) {
      error(`Could not establish forwarding connection to ${blue(counterparty.toB58String())}`)
      return
    }

    const toCounterparty = handshake<Uint8Array>(newConn.stream)

    toCounterparty.write(initiator.pubKey.marshal())

    let answer: Uint8Array | undefined
    try {
      answer = (await toCounterparty.read())?.slice()
    } catch (err) {
      // Don't catch close errors
      newConn.connection
        ?.close()
        .catch((err: any) => error(`Failed to close connection to ${blue(counterparty.toB58String())}. ${err.message}`))
      error(`Error while trying to decode answer from ${blue(counterparty.toB58String())}. Error was: ${err}`)
    }

    toCounterparty.rest()

    if (answer == null || !u8aEquals(answer, OK)) {
      // Don't catch close errors
      newConn.connection
        ?.close()
        .catch((err: any) => error(`Failed to close connection to ${blue(counterparty.toB58String())}. ${err.message}`))
      error(
        `Could not relay to ${blue(
          counterparty.toB58String()
        )} because we are unable to deliver packets. Answer was: <${yellow(
          new TextDecoder().decode(answer ?? Uint8Array.from([]))
        )}>`
      )
    }

    return toCounterparty.stream
  }
}

function getId(a: PeerId, b: PeerId) {
  const cmpResult = u8aCompare(a.pubKey.marshal(), b.pubKey.marshal())

  switch (cmpResult) {
    case 1:
      return `${a.toB58String()}${b.toB58String()}`
    case -1:
      return `${b.toB58String()}${a.toB58String()}`
    default:
      throw Error(`Invalid compare result. Loopbacks are not allowed.`)
  }
}

export default Relay
