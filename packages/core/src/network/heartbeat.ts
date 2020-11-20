import NetworkPeerStore from './network-peers'
import debug from 'debug'
import PeerId from 'peer-id'
import { EventEmitter } from 'events'
import { randomInteger, limitConcurrency } from '@hoprnet/hopr-utils'
import {
  HEARTBEAT_REFRESH_TIME,
  HEARTBEAT_INTERVAL_LOWER_BOUND,
  HEARTBEAT_INTERVAL_UPPER_BOUND,
  MAX_PARALLEL_CONNECTIONS
} from '../constants'
import { Heartbeat as HeartbeatInteraction } from '../interactions/network/heartbeat'

const log = debug('hopr-core:heartbeat')

class Heartbeat extends EventEmitter {
  timeout: any

  constructor(
    private networkPeers: NetworkPeerStore,
    private interaction: HeartbeatInteraction,
    private hangUp: (addr: PeerId) => Promise<void>
  ) {
    super()
    super.on('beat', this.connectionListener.bind(this))
  }

  connectionListener(peer: PeerId) {
    this.networkPeers.register(peer)
  }

  async checkNodes(): Promise<void> {
    const THRESHOLD_TIME = Date.now() - HEARTBEAT_REFRESH_TIME
    log(`Checking nodes older than ${THRESHOLD_TIME}`)

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
      () => !this.networkPeers.containsOlderThan(THRESHOLD_TIME),
      queryOldest
    )
  }

  setTimeout() {
    this.timeout = setTimeout(async () => {
      await this.checkNodes()
      this.setTimeout()
    }, randomInteger(HEARTBEAT_INTERVAL_LOWER_BOUND, HEARTBEAT_INTERVAL_UPPER_BOUND))
  }

  start(): void {
    this.setTimeout()
    log(`Heartbeat mechanism started`)
  }

  stop(): void {
    clearTimeout(this.timeout)
    log(`Heartbeat mechanism stopped`)
  }
}

export default Heartbeat
