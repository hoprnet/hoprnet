import type { PeerId } from '@libp2p/interface-peer-id'
import type { Connection } from '@libp2p/interface-connection'
import type { IncomingStreamData } from '@libp2p/interfaces/registrar'
import type { Initializable, Components } from '@libp2p/interfaces/components'
import type { Startable } from '@libp2p/interfaces/startable'
import type { DialOptions } from '@libp2p/interface-transport'
import type { Stream, HoprConnectOptions, HoprConnectTestingOptions } from '../types.js'
import type { ConnectComponents, ConnectInitializable } from '../components.js'

import { peerIdFromString } from '@libp2p/peer-id'

import errCode from 'err-code'
import debug from 'debug'
import chalk from 'chalk'

import { WebRTCConnection } from '../webrtc/index.js'
import { RELAY_PROTOCOLS, DELIVERY_PROTOCOLS, OK, CAN_RELAY_PROTOCOLS } from '../constants.js'
import { RelayConnection } from './connection.js'
import { RelayHandshake, RelayHandshakeMessage } from './handshake.js'
import { RelayState } from './state.js'
import { createRelayerKey, dial, DialStatus, randomInteger, retimer } from '@hoprnet/hopr-utils'
import { handshake } from 'it-handshake'

import { attemptClose } from '../utils/index.js'

const DEBUG_PREFIX = 'hopr-connect:relay'
const DEFAULT_MAX_RELAYED_CONNECTIONS = 10

const log = debug(DEBUG_PREFIX)
const error = debug(DEBUG_PREFIX.concat(':error'))
const verbose = debug(DEBUG_PREFIX.concat(':verbose'))

function printUsedRelays(peers: PeerId[], prefix = '') {
  let out = `${prefix}\n`

  for (const peer of peers) {
    if (out.length > prefix.length + 1) {
      out += `\n`
    }
    out += `  - ${peer.toString()}`
  }

  return out
}

/**
 * API interface for relayed connections
 */
class Relay implements Initializable, ConnectInitializable, Startable {
  private relayState: RelayState
  private usedRelays: PeerId[]

  private _isStarted: boolean

  private stopKeepAlive: (() => void) | undefined
  private connectedToRelays: Set<string>

  private components: Components | undefined
  private connectComponents: ConnectComponents | undefined

  constructor(private options: HoprConnectOptions, private testingOptions: HoprConnectTestingOptions) {
    this._isStarted = false

    log(`relay testing options`, testingOptions)
    this.relayState = new RelayState()

    this.options.maxRelayedConnections ??= DEFAULT_MAX_RELAYED_CONNECTIONS

    // Stores all relays that we announce to other nodes
    // to make sure we don't close these connections
    this.usedRelays = []

    // Gathers relay peer IDs the node connected
    this.connectedToRelays = new Set()

    this.onReconnect = this.onReconnect.bind(this)
    this.onDelivery = this.onDelivery.bind(this)
    this.onRelay = this.onRelay.bind(this)
    this.onCanRelay = this.onCanRelay.bind(this)
  }

  public init(components: Components) {
    this.components = components
  }

  public getComponents(): Components {
    if (this.components == null) {
      throw errCode(new Error('components not set'), 'ERR_SERVICE_MISSING')
    }

    return this.components
  }

  public initConnect(connectComponents: ConnectComponents) {
    this.connectComponents = connectComponents
  }

  public getConnectComponents(): ConnectComponents {
    if (this.connectComponents == null) {
      throw errCode(new Error('connectComponents not set'), 'ERR_SERVICE_MISSING')
    }

    return this.connectComponents
  }

  public isStarted(): boolean {
    return this._isStarted
  }

  public start(): void {}

  /**
   * Assigns the event listeners to the constructed object.
   * @dev Must not happen in the constructor because `this` is not ready
   *      at that point in time
   */
  async afterStart(): Promise<void> {
    if (!this.components) {
      throw Error(`Module has to be initialized first`)
    }

    // Requires registrar to be started first
    const protocolsDelivery = DELIVERY_PROTOCOLS(this.options.environment, this.options.supportedEnvironments)
    const protocolsRelay = RELAY_PROTOCOLS(this.options.environment, this.options.supportedEnvironments)
    const protocolsCanRelay = CAN_RELAY_PROTOCOLS(this.options.environment, this.options.supportedEnvironments)
    await this.components.getRegistrar().handle(protocolsDelivery, this.onDelivery)
    await this.components.getRegistrar().handle(protocolsRelay, this.onRelay)
    await this.components.getRegistrar().handle(protocolsCanRelay, this.onCanRelay)

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

    this._isStarted = true
  }

  /**
   * Unassigns event listeners
   */
  public stop(): void {
    if (!this._isStarted) {
      return
    }

    this.stopKeepAlive?.()
    this.connectedToRelays.clear()

    this._isStarted = false
  }

  public setUsedRelays(peers: PeerId[]) {
    log(printUsedRelays(peers, `set used relays:`))
    this.usedRelays = peers
  }

  protected async keepAliveRelayConnection(): Promise<void> {
    // TODO: perform ping as well, right now just prints out connection info
    if (this.relayState.relayedConnectionCount() > 0) {
      let outConns = `Current relay connections:\n`

      await this.relayState.forEach(async (dst) => {
        outConns += `- ${dst}\n`
      })

      // Remove occurence of last `\n`
      log(outConns.substring(0, outConns.length - 1))
    }

    let outRelays = `Currently tracked connections to relays:\n`
    for (const relayPeerId of this.connectedToRelays) {
      const countConns = this.getComponents()
        .getConnectionManager()
        .getConnections(peerIdFromString(relayPeerId)).length

      outRelays += `- ${relayPeerId}: ${countConns} connection${countConns == 1 ? '' : 's'}\n`
    }

    // Remove occurence of last `\n`
    log(outRelays.substring(0, outRelays.length - 1))
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
    const response = await dial(
      this.getComponents(),
      relay,
      RELAY_PROTOCOLS(this.options.environment, this.options.supportedEnvironments),
      false
    )

    if (response.status != DialStatus.SUCCESS) {
      error(
        `Cannot establish a connection to ${chalk.green(destination.toString())} because relay ${chalk.green(
          relay.toString()
        )} is not reachable`
      )
      return
    }

    const shaker = new RelayHandshake(response.resp.stream, this.options)

    const handshakeResult = await shaker.initiate(relay, destination)

    if (!handshakeResult.success) {
      error(`Handshake with ${relay.toString()} led to empty stream. Giving up.`)
      // Only close the connection to the relay if it does not perform relay services
      // for us.
      if (this.usedRelays.findIndex((usedRelay: PeerId) => usedRelay.equals(relay)) < 0) {
        try {
          await response.resp.conn.close()
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
    opts?: DialOptions
  ): RelayConnection | WebRTCConnection {
    const conn = new RelayConnection(
      stream,
      relay,
      destination,
      'outbound',
      this.getConnectComponents(),
      this.testingOptions,
      this.onReconnect as Relay['onReconnect']
    )

    if (!this.testingOptions.__noWebRTCUpgrade) {
      return new WebRTCConnection(conn, {
        __noWebRTCUpgrade: this.testingOptions.__noWebRTCUpgrade,
        ...opts
      })
    } else {
      return conn
    }
  }

  private upgradeInbound(initiator: PeerId, relay: PeerId, stream: Stream): RelayConnection | WebRTCConnection {
    const conn = new RelayConnection(
      stream,
      relay,
      initiator,
      'inbound',
      this.getConnectComponents(),
      this.testingOptions,
      this.onReconnect as Relay['onReconnect']
    )

    if (!this.testingOptions.__noWebRTCUpgrade) {
      return new WebRTCConnection(conn, this.testingOptions)
    } else {
      return conn
    }
  }

  /**
   * Announces to the DHT that this node is acting as a relay for the
   * given node.
   * @param node node to announce
   */
  private async announceRelayerKey(node: PeerId) {
    try {
      const key = createRelayerKey(node)

      await this.getComponents().getContentRouting().provide(key)

      log(`announced in the DHT as relayer for node ${node.toString()}`, key)
    } catch (err) {
      error(`error while attempting to provide relayer key for ${node.toString()}`)
    }
  }

  /**
   * Handles a request by a node to act as a relay.
   *
   * It creates a hanging open connection, unless the peer closes the connection
   * because the relay service is no longer needed.
   *
   * @param conn incoming connection
   */
  private async onCanRelay(conn: IncomingStreamData) {
    const shaker = handshake(conn.stream)

    try {
      // Do both operations indedepently from each other
      await Promise.all([
        // Send answer, but don't end the stream
        shaker.write(OK),
        this.announceRelayerKey(conn.connection.remotePeer)
      ])
    } catch (err) {
      error(`error in can relay protocol`, err)
      // Close the connection because it led to an error
      attemptClose(conn.connection, error)
    }
  }

  private async onRelay(conn: IncomingStreamData) {
    if (conn.connection == undefined || conn.connection.remotePeer == undefined) {
      verbose(`Received incomplete connection object`)
      return
    }

    const shaker = new RelayHandshake(conn.stream, this.options)

    log(`handling relay request from ${conn.connection.remotePeer.toString()}`)
    log(`relayed connection count: ${this.relayState.relayedConnectionCount()}`)

    try {
      if (this.relayState.relayedConnectionCount() >= (this.options.maxRelayedConnections as number)) {
        log(`relayed request rejected, already at max capacity (${this.options.maxRelayedConnections as number})`)
        await shaker.reject(RelayHandshakeMessage.FAIL_RELAY_FULL)
      } else {
        // NOTE: This cannot be awaited, otherwise it stalls the relay loop. Therefore, promise rejections must
        // be handled downstream to avoid unhandled promise rejection crashes
        await shaker.negotiate(conn.connection.remotePeer, this.getComponents(), this.relayState)
      }
    } catch (e) {
      error(`Error while processing relay request from ${conn.connection.remotePeer.toString()}: ${e}`)
    }
  }

  /**
   * Handles incoming relay requests.
   * @param conn incoming connection
   */
  private async onDelivery(conn: IncomingStreamData): Promise<void> {
    if (conn.stream == undefined || conn.connection == undefined) {
      error(
        `Dropping stream because ${conn.connection == undefined ? 'cannot determine relay address ' : ''}${
          conn.stream == undefined ? 'no stream was given' : ''
        }`
      )
      return
    }

    const handShakeResult = await new RelayHandshake(conn.stream, this.options).handle(conn.connection.remotePeer)

    if (!handShakeResult.success) {
      return
    }

    log(`incoming connection from ${handShakeResult.counterparty.toString()}`)

    const newConn = this.upgradeInbound(
      handShakeResult.counterparty,
      conn.connection.remotePeer,
      handShakeResult.stream
    )

    let upgradedConn: Connection
    try {
      // Will call internal libp2p event handler, so no further action required
      upgradedConn = await this.getComponents().getUpgrader().upgradeInbound(newConn)
    } catch (err) {
      error(`Could not upgrade relayed connection. Error was: ${err}`)
      return
    }

    // Merges all tags from `maConn` into `conn` and then make both objects
    // use the *same* array
    // This is necessary to dynamically change the connection tags once
    // a connection gets upgraded from WEBRTC_RELAYED to WEBRTC_DIRECT
    if (upgradedConn.tags == undefined) {
      upgradedConn.tags = []
    }
    upgradedConn.tags.push(...(newConn.tags as any))
    // assign the array *by value* and its entries *by reference*
    newConn.tags = upgradedConn.tags as any

    // @TODO
    // this.discovery._peerDiscovered(newConn.remotePeer, [newConn.remoteAddr])
  }

  /**
   * Called once reconnect happens
   * @param relayConn new relayed connection
   * @param counterparty counterparty of the relayed connection
   */
  private async onReconnect(relayConn: RelayConnection, counterparty: PeerId): Promise<void> {
    log(`####### inside reconnect #######`)

    let newConn: Connection

    log(`Handling reconnect attempt to ${counterparty.toString()}`)

    try {
      if (!this.testingOptions.__noWebRTCUpgrade) {
        newConn = await this.getComponents()
          .getUpgrader()
          .upgradeInbound(
            new WebRTCConnection(relayConn, this.testingOptions, {
              // libp2p interface type clash
              upgrader: this.getComponents().getUpgrader() as any
            })
          )
      } else {
        newConn = await this.getComponents().getUpgrader().upgradeInbound(relayConn)
      }
    } catch (err) {
      error(err)
      return
    }

    // @TODO remove this (1/2 done)
    this.getComponents()
      .getConnectionManager()
      // @ts-ignore not part of exposed interface (yet)
      .dialer._pendingDials?.get(counterparty.toString())
      ?.destroy()

    const existingConnections = this.getComponents().getConnectionManager().getConnections(counterparty)
    for (const existingConnection of existingConnections) {
      if (existingConnection.id === newConn.id) {
        continue
      }
      try {
        await existingConnection.close()
      } catch (err) {
        error(`Error while closing dead connection`, err)
      }
    }
  }
}

export { Relay }
