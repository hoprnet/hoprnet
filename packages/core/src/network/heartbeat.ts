import type NetworkPeerStore from './network-peers'
import type PeerId from 'peer-id'
import debug from 'debug'
import { randomInteger, limitConcurrency } from '@hoprnet/hopr-utils'
import { HEARTBEAT_INTERVAL, HEARTBEAT_INTERVAL_VARIANCE, MAX_PARALLEL_CONNECTIONS } from '../constants'
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
    const thresholdTime = Date.now() - HEARTBEAT_INTERVAL
    log(`Checking nodes since ${thresholdTime} (${new Date(thresholdTime).toLocaleString()})`)

    const toPing = this.networkPeers.pingSince(thresholdTime)

    const doPing = async (): Promise<void> => {
      await this.networkPeers.ping(toPing.pop(), async (id: PeerId) => {
        log('ping', id.toB58String())

        const pingResult = await this.interaction.interact(id).catch((err) => log('ping', err))

        if (pingResult >= 0) {
          log('ping success to', id.toB58String())
          return true
        } else {
          log('ping failed to', id.toB58String())
          await this.hangUp(id)
          return false
        }
      })
    }

    await limitConcurrency<void>(MAX_PARALLEL_CONNECTIONS, () => toPing.length <= 0, doPing)
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
