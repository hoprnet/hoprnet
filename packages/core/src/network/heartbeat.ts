import type NetworkPeerStore from './network-peers'
import type PeerId from 'peer-id'
import { randomInteger, limitConcurrency, u8aEquals, debug, retimer } from '@hoprnet/hopr-utils'
import {
  HEARTBEAT_INTERVAL,
  HEARTBEAT_TIMEOUT,
  HEARTBEAT_INTERVAL_VARIANCE,
  MAX_PARALLEL_CONNECTIONS
} from '../constants'
import { createHash, randomBytes } from 'crypto'

import type { Subscribe, SendMessage } from '../index'

const log = debug('hopr-core:heartbeat')
const error = debug('hopr-core:heartbeat:error')

const PING_HASH_ALGORITHM = 'blake2s256'

export default class Heartbeat {
  private stopHeartbeatInterval: (() => void) | undefined
  private protocolHeartbeat: string

  constructor(
    private networkPeers: NetworkPeerStore,
    subscribe: Subscribe,
    protected sendMessage: SendMessage,
    private hangUp: (addr: PeerId) => Promise<void>,
    environmentId: string
  ) {
    const errHandler = (err: any) => {
      error(`Error while processing heartbeat request`, err)
    }

    this.protocolHeartbeat = `/hopr/${environmentId}/heartbeat`

    subscribe(this.protocolHeartbeat, this.handleHeartbeatRequest.bind(this), true, errHandler)
  }

  public handleHeartbeatRequest(msg: Uint8Array, remotePeer: PeerId): Promise<Uint8Array> {
    this.networkPeers.register(remotePeer)
    log('beat')
    return Promise.resolve(Heartbeat.calculatePingResponse(msg))
  }

  public async pingNode(id: PeerId): Promise<boolean> {
    log('ping', id.toB58String())

    const challenge = randomBytes(16)
    const expectedResponse = Heartbeat.calculatePingResponse(challenge)

    try {
      const pingResponse = await this.sendMessage(id, this.protocolHeartbeat, challenge, true, {
        timeout: HEARTBEAT_TIMEOUT
      })

      if (pingResponse == null || pingResponse.length == 0 || !u8aEquals(expectedResponse, pingResponse[0])) {
        log(`Mismatched challenge. ${pingResponse}`)
        await this.hangUp(id)
        return false
      }

      log('ping success to', id.toB58String())
      return true
    } catch (e) {
      log(`Connection to ${id.toB58String()} failed: ${JSON.stringify(e)}`)
      return false
    }
  }

  private async checkNodes(): Promise<void> {
    const thresholdTime = Date.now() - HEARTBEAT_INTERVAL
    log(`Checking nodes since ${thresholdTime} (${new Date(thresholdTime).toLocaleString()})`)

    const toPing = this.networkPeers.pingSince(thresholdTime)

    const doPing = async (): Promise<void> => {
      await this.networkPeers.ping(toPing.pop(), async (id: PeerId) => await this.pingNode(id))
    }

    await limitConcurrency<void>(MAX_PARALLEL_CONNECTIONS, () => toPing.length <= 0, doPing)
    log(`finished checking nodes since ${thresholdTime} ${this.networkPeers.length()} nodes`)
    log(this.networkPeers.debugLog())
  }

  private startHeartbeatInterval() {
    const periodicCheck = async function (this: Heartbeat) {
      try {
        await this.checkNodes()
      } catch (e) {
        log('FATAL ERROR IN HEARTBEAT', e)
      }
    }.bind(this)

    this.stopHeartbeatInterval = retimer(
      periodicCheck,
      // Prevent nodes from querying each other at the very same time
      () => randomInteger(HEARTBEAT_INTERVAL, HEARTBEAT_INTERVAL + HEARTBEAT_INTERVAL_VARIANCE)
    )

    setTimeout(periodicCheck)
  }

  public start() {
    this.startHeartbeatInterval()
    log(`Heartbeat started`)
  }

  public stop() {
    this.stopHeartbeatInterval?.()
    log(`Heartbeat stopped`)
  }

  public static calculatePingResponse(challenge: Uint8Array): Uint8Array {
    return Uint8Array.from(createHash(PING_HASH_ALGORITHM).update(challenge).digest())
  }

  public async __forTestOnly_checkNodes() {
    return await this.checkNodes()
  }
}
