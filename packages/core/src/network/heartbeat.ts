import type NetworkPeers from './network-peers.js'
import type AccessControl from './access-control.js'
import type { PeerId } from '@libp2p/interface-peer-id'
import { randomInteger, u8aEquals, debug, retimer, nAtATime, u8aToHex } from '@hoprnet/hopr-utils'
import { HEARTBEAT_TIMEOUT } from '../constants.js'
import { createHash, randomBytes } from 'crypto'

import type { Subscribe, SendMessage } from '../index.js'
import EventEmitter from 'events'
import { NetworkPeersOrigin } from './network-peers.js'

const log = debug('hopr-core:heartbeat')
const error = debug('hopr-core:heartbeat:error')

const PING_HASH_ALGORITHM = 'blake2s256'

const MAX_PARALLEL_HEARTBEATS = 14
const HEARTBEAT_RUN_TIMEOUT = 2 * 60 * 1000 // 2 minutes

export type HeartbeatPingResult = {
  destination: PeerId
  lastSeen: number
}

export type HeartbeatConfig = {
  heartbeatDialTimeout: number
  heartbeatRunTimeout: number
  maxParallelHeartbeats: number
  heartbeatVariance: number
  heartbeatInterval: number
  heartbeatThreshold: number
  networkQualityThreshold: number
}

/**
 * Indicator of the current state of the P2P network
 * based on the different node types we can ping.
 */
export enum NetworkHealthIndicator {
  UNKNOWN = 'Unknown',
  RED = 'Red', // No connection, default
  ORANGE = 'Orange', // Low quality (<= 0.5) connection to at least 1 public relay
  YELLOW = 'Yellow', // High quality (> 0.5) connection to at least 1 public relay
  GREEN = 'Green' // High quality (> 0.5) connection to at least 1 public relay and 1 NAT node
}

export default class Heartbeat {
  private stopHeartbeatInterval: (() => void) | undefined
  private protocolHeartbeat: string

  private _pingNode: Heartbeat['pingNode'] | undefined

  // Initial network health is always RED
  private currentHealth: NetworkHealthIndicator = NetworkHealthIndicator.UNKNOWN

  private config: HeartbeatConfig

  constructor(
    private networkPeers: NetworkPeers,
    private subscribe: Subscribe,
    protected sendMessage: SendMessage,
    private closeConnectionsTo: (peer: PeerId) => Promise<void>,
    private reviewConnection: AccessControl['reviewConnection'],
    private stateChangeEmitter: EventEmitter,
    private isPublicNode: (addr: PeerId) => boolean,
    environmentId: string,
    config?: Partial<HeartbeatConfig>
  ) {
    this.config = {
      heartbeatDialTimeout: config?.heartbeatDialTimeout ?? HEARTBEAT_TIMEOUT,
      heartbeatRunTimeout: config?.heartbeatRunTimeout ?? HEARTBEAT_RUN_TIMEOUT,
      heartbeatInterval: config?.heartbeatInterval,
      heartbeatThreshold: config?.heartbeatThreshold,
      heartbeatVariance: config?.heartbeatVariance,
      networkQualityThreshold: config?.networkQualityThreshold,
      maxParallelHeartbeats: config?.maxParallelHeartbeats ?? MAX_PARALLEL_HEARTBEATS
    }
    this.protocolHeartbeat = `/hopr/${environmentId}/heartbeat`
  }

  private errHandler(err: any) {
    error(`Error while processing heartbeat request`, err)
  }

  public async start() {
    await this.subscribe(this.protocolHeartbeat, this.handleHeartbeatRequest.bind(this), true, this.errHandler)

    this._pingNode = this.pingNode.bind(this)
    this.startHeartbeatInterval()
    log(`Heartbeat started`)
  }

  public stop() {
    this.stopHeartbeatInterval?.()
    log(`Heartbeat stopped`)
  }

  public handleHeartbeatRequest(msg: Uint8Array, remotePeer: PeerId): Promise<Uint8Array> {
    if (this.networkPeers.has(remotePeer)) {
      this.networkPeers.updateRecord({
        destination: remotePeer,
        lastSeen: Date.now()
      })
    } else {
      this.networkPeers.register(remotePeer, NetworkPeersOrigin.INCOMING_CONNECTION)
    }

    // Recalculate network health when incoming heartbeat has been received
    this.recalculateNetworkHealth()

    log(`received heartbeat from ${remotePeer.toString()}`)
    return Promise.resolve(Heartbeat.calculatePingResponse(msg))
  }

  /**
   * Attempts to ping another peer.
   * @param destination id of the node to ping
   * @param signal [optional] abort controller to prematurely end request
   * @returns a Promise of a pingResult object with property `lastSeen < 0` if there were a timeout
   */
  public async pingNode(destination: PeerId, signal?: AbortSignal): Promise<HeartbeatPingResult> {
    log(`ping ${destination.toString()} (timeout ${this.config.heartbeatDialTimeout})`)

    const origin = this.networkPeers.has(destination)
      ? this.networkPeers.getConnectionInfo(destination).origin
      : NetworkPeersOrigin.OUTGOING_CONNECTION
    const allowed = await this.reviewConnection(destination, origin)
    if (!allowed) throw Error('Connection to node is not allowed')

    const challenge = randomBytes(16)
    let pingResponse: Uint8Array[] | undefined

    try {
      pingResponse = await this.sendMessage(destination, this.protocolHeartbeat, challenge, true, {
        timeout: this.config.heartbeatDialTimeout,
        signal
      })
    } catch (err) {
      log(`Connection to ${destination.toString()} failed: ${err?.message}`)
      return {
        destination,
        lastSeen: -1
      }
    }

    const expectedResponse = Heartbeat.calculatePingResponse(challenge)

    if (pingResponse == null || pingResponse.length != 1 || !u8aEquals(expectedResponse, pingResponse[0])) {
      log(`Mismatched challenge. Got ${u8aToHex(pingResponse[0])} but expected ${u8aToHex(expectedResponse)}`)

      await this.closeConnectionsTo(destination)

      return {
        destination,
        lastSeen: -1
      }
    }

    return {
      destination,
      lastSeen: Date.now()
    }
  }

  /**
   * Recalculates the network health indicator based on the
   * current network state knowledge.
   * @returns Value of the current network health indicator (possibly updated).
   */
  public recalculateNetworkHealth(): NetworkHealthIndicator {
    let newHealthValue = NetworkHealthIndicator.RED
    let lowQualityPublic = 0
    let lowQualityNonPublic = 0
    let highQualityPublic = 0
    let highQualityNonPublic = 0

    // Count quality of public/non-public nodes
    for (let entry of this.networkPeers.getAllEntries()) {
      let quality = this.networkPeers.qualityOf(entry.id)
      if (this.isPublicNode(entry.id)) {
        quality > this.config.networkQualityThreshold ? ++highQualityPublic : ++lowQualityPublic
      } else {
        quality > this.config.networkQualityThreshold ? ++highQualityNonPublic : ++lowQualityNonPublic
      }
    }

    // ORANGE state = low quality connection to any node
    if (lowQualityPublic > 0) newHealthValue = NetworkHealthIndicator.ORANGE

    // YELLOW = high-quality connection to a public node
    if (highQualityPublic > 0) newHealthValue = NetworkHealthIndicator.YELLOW

    // GREEN = hiqh-quality connection to a public and a non-public node
    if (highQualityPublic > 0 && highQualityNonPublic > 0) newHealthValue = NetworkHealthIndicator.GREEN

    // Emit network health change event if needed
    if (newHealthValue != this.currentHealth) {
      let oldValue = this.currentHealth
      this.currentHealth = newHealthValue
      this.stateChangeEmitter.emit('hopr:network-health-changed', oldValue, this.currentHealth)
    }

    return this.currentHealth
  }

  /**
   * Performs a ping request to all nodes who were not seen since the threshold
   */
  protected async checkNodes(): Promise<void> {
    const thresholdTime = Date.now() - this.config.heartbeatThreshold
    log(`Checking nodes since ${thresholdTime} (${new Date(thresholdTime).toLocaleString()})`)

    const abort = new AbortController()

    let finished = false

    setTimeout(() => {
      if (!finished) {
        abort.abort()
      }
    }, this.config.heartbeatRunTimeout).unref()

    // Create an object that describes which work has to be done
    // by the workers, i.e. the pingNode code
    const pingWork = this.networkPeers
      .pingSince(thresholdTime)
      .map<[destination: PeerId, signal: AbortSignal]>((peerToPing: PeerId) => [peerToPing, abort.signal])

    const start = Date.now()
    const pingResults = await nAtATime(this._pingNode, pingWork, this.config.maxParallelHeartbeats)

    finished = true

    for (const [resultIndex, pingResult] of pingResults.entries()) {
      if (pingResult instanceof Error) {
        // we need to get the destination so we can map a ping error properly
        const [destination, _abortSignal] = pingWork[resultIndex]
        const failedPingResult = {
          destination,
          lastSeen: -1
        }
        this.networkPeers.updateRecord(failedPingResult)
      } else {
        this.networkPeers.updateRecord(pingResult)
      }
    }

    // Recalculate the network health indicator state after checking nodes
    this.recalculateNetworkHealth()

    log(
      this.networkPeers.debugLog(
        `finished checking ${pingWork.length} node${pingWork.length == 1 ? '' : 's'} since ${thresholdTime} within ${
          Date.now() - start
        } ms`
      )
    )
  }

  /**
   * Starts the periodic check
   */
  private startHeartbeatInterval() {
    const periodicCheck = async function (this: Heartbeat) {
      try {
        await this.checkNodes()
      } catch (err) {
        log('FATAL ERROR IN HEARTBEAT', err)
      }
    }.bind(this)

    this.stopHeartbeatInterval = retimer(
      periodicCheck,
      // Prevent nodes from querying each other at the very same time
      () => randomInteger(this.config.heartbeatInterval, this.config.heartbeatInterval + this.config.heartbeatVariance)
    )
  }

  public static calculatePingResponse(challenge: Uint8Array): Uint8Array {
    return Uint8Array.from(createHash(PING_HASH_ALGORITHM).update(challenge).digest())
  }
}
