import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'
import NetworkPeerStore from './peerStore'
import debug from 'debug'
import { getTokens, Token } from '../utils'
import PeerId from 'peer-id'
import { Entry } from './peerStore'
import { EventEmitter } from 'events'
import PeerInfo from 'peer-info'
import { randomInteger } from '@hoprnet/hopr-utils'
import {HEARTBEAT_REFRESH_TIME, HEARTBEAT_INTERVAL_LOWER_BOUND, HEARTBEAT_INTERVAL_UPPER_BOUND, MAX_PARALLEL_CONNECTIONS} from '../constants'


const log = debug('hopr-core:heartbeat')


class Heartbeat<Chain extends HoprCoreConnector> extends EventEmitter {
  timeout: any

  constructor(
    public node: Hopr<Chain>,
    private networkPeers: NetworkPeerStore
    ) {

    super()

    this.node.on('peer:connect', this.connectionListener.bind(this))
    super.on('beat', this.connectionListener.bind(this))
  }

  private connectionListener(peer: PeerId | PeerInfo) {
    const peerIdString = (PeerId.isPeerId(peer) ? peer : peer.id).toB58String()

    this.networkPeers.push({
      id: peerIdString,
      lastSeen: Date.now(),
    })
  }

  async checkNodes(): Promise<void> {

    log(`Checking nodes`)
    this.networkPeers.debugLog()

    const promises: Promise<void>[] = Array.from({ length: MAX_PARALLEL_CONNECTIONS })
    const tokens = getTokens(MAX_PARALLEL_CONNECTIONS)

    const THRESHOLD_TIME = Date.now() - HEARTBEAT_REFRESH_TIME

    const queryNode = async (peer: string, token: Token): Promise<void> => {
      while (
        tokens.length > 0 &&
        this.networkPeers.updatedSince(THRESHOLD_TIME)
      ) {
        let nextPeer = this.networkPeers.pop()
        let token = tokens.pop() as Token

        promises[token] = queryNode(nextPeer.id, token)
      }

      let currentPeerId: PeerId

      while (true) {
        currentPeerId = PeerId.createFromB58String(peer)

        try {
          await this.node.interactions.network.heartbeat.interact(currentPeerId)

          this.networkPeers.push({
            id: peer,
            lastSeen: Date.now(),
          })
        } catch (err) {
          await this.node.hangUp(currentPeerId)
          this.networkPeers.blacklistPeer(peer)

          // ONLY FOR TESTING
          log(`Deleted node ${peer}`)
          this.networkPeers.debugLog()
          // END ONLY FOR TESTING
        }

        if (this.networkPeers.updatedSince(THRESHOLD_TIME)) {
          peer = this.networkPeers.pop().id
        } else {
          break
        }
      }

      promises[token] = undefined
      tokens.push(token)
    }

    if (this.networkPeers.updatedSince(THRESHOLD_TIME)) {
      let token = tokens.pop() as Token
      promises[token] = queryNode(this.networkPeers.pop().id, token)
    }

    await Promise.all(promises)
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
