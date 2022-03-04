import type NetworkPeerStore from './network-peers'
import type PeerId from 'peer-id'
import { randomInteger, u8aEquals, debug, retimer, nAtATime, u8aToHex } from '@hoprnet/hopr-utils'
import { HEARTBEAT_INTERVAL, HEARTBEAT_TIMEOUT, HEARTBEAT_INTERVAL_VARIANCE } from '../constants'
import { createHash, randomBytes } from 'crypto'

import type { Subscribe, SendMessage } from '../index'

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
}

export default class Heartbeat {
  private stopHeartbeatInterval: (() => void) | undefined
  private protocolHeartbeat: string

  private _pingNode: Heartbeat['pingNode'] | undefined

  private config: HeartbeatConfig

  constructor(
    private networkPeers: NetworkPeerStore,
    subscribe: Subscribe,
    protected sendMessage: SendMessage,
    private hangUp: (addr: PeerId) => Promise<void>,
    environmentId: string,
    config?: Partial<HeartbeatConfig>
  ) {
    this.config = {
      heartbeatDialTimeout: config?.heartbeatDialTimeout ?? HEARTBEAT_TIMEOUT,
      heartbeatRunTimeout: config?.heartbeatRunTimeout ?? HEARTBEAT_RUN_TIMEOUT,
      heartbeatInterval: config?.heartbeatInterval ?? HEARTBEAT_INTERVAL,
      heartbeatThreshold: config?.heartbeatThreshold ?? HEARTBEAT_INTERVAL,
      heartbeatVariance: config?.heartbeatVariance ?? HEARTBEAT_INTERVAL_VARIANCE,
      maxParallelHeartbeats: config?.maxParallelHeartbeats ?? MAX_PARALLEL_HEARTBEATS
    }
    const errHandler = (err: any) => {
      error(`Error while processing heartbeat request`, err)
    }

    this.protocolHeartbeat = `/hopr/${environmentId}/heartbeat`

    await subscribe(this.protocolHeartbeat, this.handleHeartbeatRequest.bind(this), true, errHandler)
  }

  public start() {
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
      this.networkPeers.register(remotePeer)
    }

    log(`received heartbeat from ${remotePeer.toB58String()}`)
    return Promise.resolve(Heartbeat.calculatePingResponse(msg))
  }

  /**
   * Attempts to ping another peer.
   * @param destination id of the node to ping
   * @param signal [optional] abort controller to prematurely end request
   * @returns a Promise of a pingResult object with property `lastSeen < 0` if there were a timeout
   */
  public async pingNode(destination: PeerId, signal?: AbortSignal): Promise<HeartbeatPingResult> {
    log('ping', destination.toB58String())

    const challenge = randomBytes(16)
    let pingResponse: Uint8Array[] | undefined

    try {
      pingResponse = await this.sendMessage(destination, this.protocolHeartbeat, challenge, true, {
        timeout: this.config.heartbeatDialTimeout,
        signal
      })
    } catch (err) {
      log(`Connection to ${destination.toB58String()} failed: ${err?.message}`)
      return {
        destination,
        lastSeen: -1
      }
    }

    const expectedResponse = Heartbeat.calculatePingResponse(challenge)

    if (pingResponse == null || pingResponse.length != 1 || !u8aEquals(expectedResponse, pingResponse[0])) {
      log(`Mismatched challenge. Got ${u8aToHex(pingResponse[0])} but expected ${u8aToHex(expectedResponse)}`)
      await this.hangUp(destination)

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
    }, this.config.heartbeatRunTimeout)

    // Create an object that describes which work has to be done
    // by the workers, i.e. the pingNode code
    const pingWork = this.networkPeers
      .pingSince(thresholdTime)
      .map<[destination: PeerId, signal: AbortSignal]>((peerToPing: PeerId) => [peerToPing, abort.signal])

    const start = Date.now()
    const pingResults = await nAtATime(this._pingNode, pingWork, this.config.maxParallelHeartbeats)

    log(`Heartbeat run pinging ${pingWork.length} nodes took ${Date.now() - start} ms`)

    finished = true

    for (const pingResult of pingResults) {
      // Filter unexpected network errors
      if (pingResult instanceof Error) {
        continue
      }
      this.networkPeers.updateRecord(pingResult)
    }

    log(`finished checking nodes since ${thresholdTime} ${this.networkPeers.length()} nodes`)
    log(this.networkPeers.debugLog())
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
