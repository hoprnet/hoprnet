import type NetworkPeers from './network-peers.js'
import type { PeerId } from '@libp2p/interface-peer-id'
import { randomInteger, u8aEquals, debug, retimer, nAtATime, u8aToHex, pickVersion } from '@hoprnet/hopr-utils'
import { createHash, randomBytes } from 'crypto'

import type { Subscribe, SendMessage } from '../index.js'
import { NetworkPeersOrigin } from './network-peers.js'

const log = debug('hopr-core:heartbeat')
const error = debug('hopr-core:heartbeat:error')

// Do not type-check JSON files
// @ts-ignore
import pkg from '../../package.json' assert { type: 'json' }

const NORMALIZED_VERSION = pickVersion(pkg.version)

const PING_HASH_ALGORITHM = 'blake2s256'

const MAX_PARALLEL_HEARTBEATS = 14

export type HeartbeatPingResult = {
  destination: PeerId
  lastSeen: number
}

export type HeartbeatConfig = {
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
  private protocolHeartbeat: string | string[]

  // Initial network health is always RED
  private currentHealth: NetworkHealthIndicator = NetworkHealthIndicator.UNKNOWN

  private config: HeartbeatConfig

  constructor(
    private networkPeers: NetworkPeers,
    private subscribe: Subscribe,
    protected sendMessage: SendMessage,
    private closeConnectionsTo: (peer: PeerId) => void,
    private onNetworkHealthChange: (oldValue: NetworkHealthIndicator, currentHealth: NetworkHealthIndicator) => void,
    private isPublicNode: (addr: PeerId) => boolean,
    environmentId: string,
    config?: Partial<HeartbeatConfig>
  ) {
    this.config = {
      heartbeatInterval: config?.heartbeatInterval,
      heartbeatThreshold: config?.heartbeatThreshold,
      heartbeatVariance: config?.heartbeatVariance,
      networkQualityThreshold: config?.networkQualityThreshold,
      maxParallelHeartbeats: config?.maxParallelHeartbeats ?? MAX_PARALLEL_HEARTBEATS
    }
    this.protocolHeartbeat = [
      // current
      `/hopr/${environmentId}/heartbeat/${NORMALIZED_VERSION}`,
      // deprecated
      `/hopr/${environmentId}/heartbeat`
    ]

    this.pingNode = this.pingNode.bind(this)
  }

  private errHandler(err: any) {
    error(`Error while processing heartbeat request`, err)
  }

  public async start() {
    await this.subscribe(this.protocolHeartbeat, this.handleHeartbeatRequest.bind(this), true, this.errHandler)

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
   * @returns a Promise of a pingResult object with property `lastSeen < 0` if there were a timeout
   */
  public async pingNode(destination: PeerId): Promise<HeartbeatPingResult> {
    log(`ping ${destination.toString()}`)

    const challenge = randomBytes(16)
    let pingResponse: Uint8Array[] | undefined

    // Dial attempt will fail if `destination` is not registered
    let pingError: any
    let pingErrorThrown = false
    try {
      pingResponse = await this.sendMessage(destination, this.protocolHeartbeat, challenge, true)
    } catch (err) {
      pingErrorThrown = true
      pingError = err
    }

    if (pingErrorThrown || pingResponse == null || pingResponse.length != 1) {
      if (pingErrorThrown) {
        log(`Error while pinging ${destination.toString()}`, pingError)
      } else if (pingResponse == null || pingResponse.length == 1) {
        const expectedResponse = Heartbeat.calculatePingResponse(challenge)

        if (!u8aEquals(expectedResponse, pingResponse[0])) {
          log(`Mismatched challenge. Got ${u8aToHex(pingResponse[0])} but expected ${u8aToHex(expectedResponse)}`)
        }

        // Eventually close the connections, all errors are handled
        this.closeConnectionsTo(destination)
      }

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
      this.onNetworkHealthChange(oldValue, this.currentHealth)
    }

    return this.currentHealth
  }

  /**
   * Performs a ping request to all nodes who were not seen since the threshold
   */
  protected async checkNodes(): Promise<void> {
    const thresholdTime = Date.now() - this.config.heartbeatThreshold
    log(`Checking nodes since ${thresholdTime} (${new Date(thresholdTime).toLocaleString()})`)

    // Create an object that describes which work has to be done
    // by the workers, i.e. the pingNode code
    const pingWork = this.networkPeers
      .pingSince(thresholdTime)
      .map<[destination: PeerId]>((peerToPing: PeerId) => [peerToPing])

    const start = Date.now()

    // Will handle timeouts automatically
    const pingResults = await nAtATime(this.pingNode, pingWork, this.config.maxParallelHeartbeats)

    for (const [resultIndex, pingResult] of pingResults.entries()) {
      if (pingResult instanceof Error) {
        // we need to get the destination so we can map a ping error properly
        const [destination] = pingWork[resultIndex]
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
