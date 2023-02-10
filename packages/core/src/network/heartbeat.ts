import type NetworkPeers from './network-peers.js'
import type { PeerId } from '@libp2p/interface-peer-id'

import {
  randomInteger,
  u8aEquals,
  debug,
  retimer,
  timeout,
  nAtATime,
  u8aToHex,
  pickVersion,
  create_gauge,
  create_counter,
  create_histogram_with_buckets,
  create_multi_gauge
} from '@hoprnet/hopr-utils'
import type { Components } from '@libp2p/interfaces/components'

import { randomBytes } from 'crypto'

import type { SendMessage } from '../index.js'
import { NetworkPeersOrigin } from './network-peers.js'
import { pipe } from 'it-pipe'
import { reply_to_ping, generate_ping_response } from '../../lib/core_misc.js'

const log = debug('hopr-core:heartbeat')
const error = debug('hopr-core:heartbeat:error')

// Do not type-check JSON files
// @ts-ignore
import pkg from '../../package.json' assert { type: 'json' }

const NORMALIZED_VERSION = pickVersion(pkg.version)

const MAX_PARALLEL_HEARTBEATS = 14

const HEART_BEAT_ROUND_TIMEOUT = 60000 // 1 minute in ms;

// Metrics
const metric_networkHealth = create_gauge('core_gauge_network_health', 'Connectivity health indicator')
const metric_timeToHeartbeat = create_histogram_with_buckets(
  'core_histogram_heartbeat_time_seconds',
  'Measures total time it takes to probe all other nodes (in seconds)',
  new Float64Array([0.5, 1.0, 2.5, 5, 10.0, 15.0, 30.0, 60.0, 90.0, 120.0, 300.0])
)
const metric_timeToPing = create_histogram_with_buckets(
  'core_histogram_ping_time_seconds',
  'Measures total time it takes to ping a single node (seconds)',
  new Float64Array([0.5, 1.0, 2.5, 5, 10.0, 15.0, 30.0, 60.0, 90.0, 120.0, 300.0])
)

const metric_pingSuccessCount = create_counter(
  'core_counter_heartbeat_successful_pings',
  'Total number of successful pings'
)
const metric_pingFailureCount = create_counter('core_counter_heartbeat_failed_pings', 'Total number of failed pings')

const metric_peersByQuality = create_multi_gauge(
  'core_mgauge_peers_by_quality',
  'Number different peer types by quality',
  ['type', 'quality']
)
const metric_peers = create_gauge('core_gauge_num_peers', 'Number of all peers')

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

  // Initial network health is UNKNOWN
  private currentHealth: NetworkHealthIndicator = NetworkHealthIndicator.UNKNOWN

  private config: HeartbeatConfig

  constructor(
    private me: PeerId,
    private networkPeers: NetworkPeers,
    private libp2pComponents: Components,
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
    this.libp2pComponents.getRegistrar().handle(this.protocolHeartbeat, async ({ connection, stream }) => {
      if (this.networkPeers.has(connection.remotePeer)) {
        this.networkPeers.updateRecord({
          destination: connection.remotePeer,
          lastSeen: Date.now()
        })
      } else {
        this.networkPeers.register(connection.remotePeer, NetworkPeersOrigin.INCOMING_CONNECTION)
      }

      this.recalculateNetworkHealth()

      try {
        await pipe(
          stream.source,
          async function* pipeToHandler(source: AsyncIterable<Uint8Array>) {
            yield* {
              [Symbol.asyncIterator]() {
                return reply_to_ping(source[Symbol.asyncIterator]())
              }
            }
          },
          stream.sink
        )
      } catch (err) {
        this.errHandler(err)
      }
    })

    this.startHeartbeatInterval()
    log(`Heartbeat started`)
  }

  public stop() {
    this.stopHeartbeatInterval?.()
    log(`Heartbeat stopped`)
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

    const ping_timer = metric_timeToPing.start_measure()
    // Dial attempt will fail if `destination` is not registered
    let pingError: any
    let pingErrorThrown = false
    try {
      // race the HEART_BEAT_ROUND_TIMEOUT. Abort ping action when timeout.
      pingResponse = await timeout(HEART_BEAT_ROUND_TIMEOUT, () =>
        this.sendMessage(destination, this.protocolHeartbeat, challenge, true)
      )
    } catch (err) {
      pingErrorThrown = true
      pingError = err
    }

    if (pingErrorThrown || pingResponse == null || pingResponse.length != 1) {
      if (pingErrorThrown) {
        log(`Error while pinging ${destination.toString()}`, pingError)
      } else if (pingResponse == null || pingResponse.length == 1) {
        const expectedResponse = generate_ping_response(
          new Uint8Array(challenge.buffer, challenge.byteOffset, challenge.length)
        )

        if (!u8aEquals(expectedResponse, pingResponse[0])) {
          log(`Mismatched challenge. Got ${u8aToHex(pingResponse[0])} but expected ${u8aToHex(expectedResponse)}`)
        }

        // Eventually close the connections, all errors are handled
        this.closeConnectionsTo(destination)
      }

      metric_timeToPing.cancel_measure(ping_timer)
      metric_pingFailureCount.increment()
      return {
        destination,
        lastSeen: -1
      }
    }

    metric_timeToPing.record_measure(ping_timer)
    metric_pingSuccessCount.increment()
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
    let newHealthValue: NetworkHealthIndicator = NetworkHealthIndicator.RED
    let lowQualityPublic = 0
    let lowQualityNonPublic = 0
    let highQualityPublic = 0
    let highQualityNonPublic = 0

    // Count quality of public/non-public nodes
    for (let entry of this.networkPeers.getAllEntries()) {
      let quality = this.networkPeers.qualityOf(entry.id)
      if (this.isPublicNode(entry.id)) {
        if (quality > this.config.networkQualityThreshold) {
          ++highQualityPublic
        } else {
          ++lowQualityPublic
        }
      } else {
        if (quality > this.config.networkQualityThreshold) {
          ++highQualityNonPublic
        } else {
          ++lowQualityNonPublic
        }
      }
    }

    // ORANGE state = low quality connection to any node
    if (lowQualityPublic > 0) newHealthValue = NetworkHealthIndicator.ORANGE

    // YELLOW = high-quality connection to a public node
    if (highQualityPublic > 0) newHealthValue = NetworkHealthIndicator.YELLOW

    // GREEN = hiqh-quality connection to a public and a non-public node OR we're public node
    if (highQualityPublic > 0 && (this.isPublicNode(this.me) || highQualityNonPublic > 0))
      newHealthValue = NetworkHealthIndicator.GREEN

    log(
      `network health details: ${lowQualityPublic} LQ public, ${lowQualityNonPublic} LQ non-public, ${highQualityPublic} HQ public, ${highQualityNonPublic} HQ non-public`
    )

    metric_peersByQuality.set(['public', 'low'], lowQualityPublic)
    metric_peersByQuality.set(['public', 'high'], highQualityPublic)
    metric_peersByQuality.set(['nonPublic', 'low'], lowQualityNonPublic)
    metric_peersByQuality.set(['nonPublic', 'high'], highQualityNonPublic)

    // Emit network health change event if needed
    if (newHealthValue != this.currentHealth) {
      let oldValue = this.currentHealth
      this.currentHealth = newHealthValue
      this.onNetworkHealthChange(oldValue, this.currentHealth)

      // Map network state to integers
      switch (newHealthValue as NetworkHealthIndicator) {
        case NetworkHealthIndicator.UNKNOWN:
          metric_networkHealth.set(0)
          break
        case NetworkHealthIndicator.RED:
          metric_networkHealth.set(1)
          break
        case NetworkHealthIndicator.ORANGE:
          metric_networkHealth.set(2)
          break
        case NetworkHealthIndicator.YELLOW:
          metric_networkHealth.set(3)
          break
        case NetworkHealthIndicator.GREEN:
          metric_networkHealth.set(4)
          break
      }
    }

    metric_peers.set(highQualityPublic + lowQualityPublic + highQualityNonPublic + highQualityPublic)

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
    const metric_timer = metric_timeToHeartbeat.start_measure()
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

    metric_timeToHeartbeat.record_measure(metric_timer)

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
}
