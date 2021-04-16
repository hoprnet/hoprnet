import type NetworkPeerStore from './network-peers'
import type PeerId from 'peer-id'
import { Logger, Hash } from '@hoprnet/hopr-utils'
import { randomInteger, limitConcurrency, LibP2PHandlerFunction, u8aEquals, DialOpts } from '@hoprnet/hopr-utils'
import { HEARTBEAT_INTERVAL, HEARTBEAT_INTERVAL_VARIANCE, MAX_PARALLEL_CONNECTIONS } from '../constants'
import { PROTOCOL_HEARTBEAT, HEARTBEAT_TIMEOUT } from '../constants'
import { randomBytes } from 'crypto'

const log = Logger.getLogger('hopr-core.heartbeat')

export default class Heartbeat {
  private timeout: NodeJS.Timeout

  constructor(
    private networkPeers: NetworkPeerStore,
    subscribe: (protocol: string, handler: LibP2PHandlerFunction, includeReply: boolean) => void,
    private sendMessageAndExpectResponse: (
      dst: PeerId,
      proto: string,
      msg: Uint8Array,
      opts: DialOpts
    ) => Promise<Uint8Array>,
    private hangUp: (addr: PeerId) => Promise<void>
  ) {
    subscribe(PROTOCOL_HEARTBEAT, this.handleHeartbeatRequest.bind(this), true)
  }

  public handleHeartbeatRequest(msg: Uint8Array, remotePeer: PeerId): Uint8Array {
    log.debug('beat')
    this.networkPeers.register(remotePeer)
    return Hash.create(msg).serialize()
  }

  public async pingNode(id: PeerId): Promise<boolean> {
    log.info('ping', id.toB58String())

    const challenge = randomBytes(16)
    const expectedResponse = Hash.create(challenge).serialize()

    try {
      const pingResponse = await this.sendMessageAndExpectResponse(id, PROTOCOL_HEARTBEAT, challenge, {
        timeout: HEARTBEAT_TIMEOUT
      })

      if (pingResponse == null || !u8aEquals(expectedResponse, pingResponse)) {
        log.warn(`Mismatched challenge. ${pingResponse}`)
        await this.hangUp(id)
        return false
      }

      log.info('ping success to', id.toB58String())
      return true
    } catch (e) {
      log.error(`Connection to ${id.toB58String()} failed: ${e}`)
      return false
    }
  }

  private async checkNodes(): Promise<void> {
    const thresholdTime = Date.now() - HEARTBEAT_INTERVAL
    log.debug(`Checking nodes since ${thresholdTime} (${new Date(thresholdTime).toLocaleString()})`)

    const toPing = this.networkPeers.pingSince(thresholdTime)

    const doPing = async (): Promise<void> => {
      await this.networkPeers.ping(toPing.pop(), async (id: PeerId) => await this.pingNode(id))
    }

    await limitConcurrency<void>(MAX_PARALLEL_CONNECTIONS, () => toPing.length <= 0, doPing)
    // TODO improve this
    // We want to log to INFO level when there is a change from before,
    // else we should log to DEBUG, in order not to flood the logging in production
    log.info(this.networkPeers.debugLog())
  }

  private tick() {
    this.timeout = setTimeout(async () => {
      await this.checkNodes()
      this.tick()
    }, randomInteger(HEARTBEAT_INTERVAL, HEARTBEAT_INTERVAL + HEARTBEAT_INTERVAL_VARIANCE))
  }

  public start() {
    log.debug(`Heartbeat started`)
    this.tick()
  }

  public stop() {
    log.debug(`Heartbeat stopped`)
    clearTimeout(this.timeout)
  }

  public async __forTestOnly_checkNodes() {
    return await this.checkNodes()
  }
}
