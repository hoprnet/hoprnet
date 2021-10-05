import type NetworkPeerStore from './network-peers'
import type PeerId from 'peer-id'
import debug from 'debug'
import { Hash } from '@hoprnet/hopr-utils'
import { randomInteger, limitConcurrency, LibP2PHandlerFunction, u8aEquals, DialOpts } from '@hoprnet/hopr-utils'
import { HEARTBEAT_INTERVAL, HEARTBEAT_INTERVAL_VARIANCE, MAX_PARALLEL_CONNECTIONS } from '../constants'
import { PROTOCOL_HEARTBEAT, HEARTBEAT_TIMEOUT } from '../constants'
import { randomBytes } from 'crypto'

const log = debug('hopr-core:heartbeat')

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
    ) => Promise<Uint8Array[]>,
    private hangUp: (addr: PeerId) => Promise<void>
  ) {
    subscribe(PROTOCOL_HEARTBEAT, this.handleHeartbeatRequest.bind(this), true)
  }

  public handleHeartbeatRequest(msg: Uint8Array, remotePeer: PeerId): Uint8Array {
    this.networkPeers.register(remotePeer)
    log('beat')
    return Hash.create(msg).serialize()
  }

  public async pingNode(id: PeerId): Promise<boolean> {
    log('ping', id.toB58String())

    const challenge = randomBytes(16)
    const expectedResponse = Hash.create(challenge).serialize()

    try {
      const pingResponse = await this.sendMessageAndExpectResponse(id, PROTOCOL_HEARTBEAT, challenge, {
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

  private tick() {
    this.timeout = setTimeout(async () => {
      try {
        await this.checkNodes()
      } catch (e) {
        log('FATAL ERROR IN HEARTBEAT', e)
      }
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
