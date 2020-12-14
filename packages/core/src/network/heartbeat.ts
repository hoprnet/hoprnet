import type NetworkPeerStore from './network-peers'
import type PeerId from 'peer-id'
import debug from 'debug'
import { randomInteger, limitConcurrency } from '@hoprnet/hopr-utils'
import {
  HEARTBEAT_REFRESH,
  HEARTBEAT_INTERVAL,
  HEARTBEAT_INTERVAL_VARIANCE,
  MAX_PARALLEL_CONNECTIONS
} from '../constants'
import { Heartbeat as HeartbeatInteraction } from '../interactions/network/heartbeat'

const log = debug('hopr-core:heartbeat')

export default class Heartbeat {
  private timeout: NodeJS.Timeout

  constructor(
    private networkPeers: NetworkPeerStore,
    private interaction: HeartbeatInteraction,
    private hangUp: (addr: PeerId) => Promise<void>
  ) {}

  private async checkNodes(): Promise<void> {
    const thresholdTime = Date.now() - HEARTBEAT_REFRESH
    log(`Checking nodes older than ${new Date(thresholdTime).toLocaleString()}`)

    const queryOldest = async (): Promise<void> => {
      await this.networkPeers.pingOldest(async (id: PeerId) => {
        log('ping', id.toB58String())
        try {
          await this.interaction.interact(id)
          log('ping success to', id.toB58String())
          return true
        } catch (err) {
          log('ping failed to', id.toB58String(), err)
          await this.hangUp(id)
          return false
        }
      })
    }

    await limitConcurrency<void>(
      MAX_PARALLEL_CONNECTIONS,
      () => !this.networkPeers.containsOlderThan(thresholdTime),
      queryOldest
    )
    log(this.networkPeers.debugLog())
  }

  private tick() {
    this.timeout = setTimeout(async () => {
      await this.checkNodes()
      this.tick()
    }, randomInteger(HEARTBEAT_INTERVAL, HEARTBEAT_INTERVAL + HEARTBEAT_INTERVAL_VARIANCE))
  }

  public start() {
    this.tick()
    log(`Heartbeat started`)
  }

  public stop() {
    clearTimeout(this.timeout)
    log(`Heartbeat stopped`)
  }

  public async __forTestOnly_checkNodes() {
    return await this.checkNodes()
  }
}
