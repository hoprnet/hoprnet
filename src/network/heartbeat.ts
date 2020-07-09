import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'

import debug from 'debug'
const log = debug('hopr-core:heartbeat')

import { getTokens, Token } from '../utils'

import PeerId from 'peer-id'

import { Entry } from './peerStore'
import { EventEmitter } from 'events'
import PeerInfo from 'peer-info'
import { randomInteger } from '@hoprnet/hopr-utils'

const REFRESH_TIME = 103 * 1000
const CHECK_INTERVAL_LOWER_BOUND = 41 * 1000
const CHECK_INTERVAL_UPPER_BOUND = 59 * 1000

const MAX_PARALLEL_CONNECTIONS = 10

class Heartbeat<Chain extends HoprCoreConnector> extends EventEmitter {
  interval: any
  timeout: any

  constructor(public node: Hopr<Chain>) {
    super()

    this.node.on('peer:connect', this.connectionListener.bind(this))
    super.on('beat', this.connectionListener.bind(this))
  }

  private connectionListener(peer: PeerId | PeerInfo) {
    const peerIdString = (PeerId.isPeerId(peer) ? peer : peer.id).toB58String()

    this.node.network.peerStore.push({
      id: peerIdString,
      lastSeen: Date.now(),
    })
  }

  async checkNodes(): Promise<void> {
    log(`Checking nodes`)
    this.node.network.peerStore.peers.forEach((entry) => log(entry.id))
    const promises: Promise<void>[] = Array.from({ length: MAX_PARALLEL_CONNECTIONS })
    const tokens = getTokens(MAX_PARALLEL_CONNECTIONS)

    const THRESHOLD_TIME = Date.now() - REFRESH_TIME

    const queryNode = async (peer: string, token: Token): Promise<void> => {
      let nextToken: number
      let nextPeer: Entry

      while (
        tokens.length > 0 &&
        this.node.network.peerStore.peers.length > 0 &&
        this.node.network.peerStore.top(1)[0].lastSeen < THRESHOLD_TIME
      ) {
        nextPeer = this.node.network.peerStore.pop()

        nextToken = tokens.pop() as Token

        if (promises[nextToken] != null) {
          promises[nextToken].then(() => queryNode(nextPeer.id, nextToken))
        } else {
          promises[nextToken] = queryNode(nextPeer.id, nextToken)
        }
      }

      let currentPeerId: PeerId

      while (true) {
        currentPeerId = PeerId.createFromB58String(peer)

        try {
          await this.node.interactions.network.heartbeat.interact(currentPeerId)

          this.node.network.peerStore.push({
            id: peer,
            lastSeen: Date.now(),
          })
        } catch (err) {
          await this.node.hangUp(currentPeerId)

          this.node.network.peerStore.blacklistPeer(peer)

          // ONLY FOR TESTING
          log(`Deleted node ${peer}`)
          log(`current nodes:`)
          this.node.network.peerStore.peers.forEach((node: Entry) => log(node.id))
          // END ONLY FOR TESTING
        }

        if (
          this.node.network.peerStore.peers.length > 0 &&
          this.node.network.peerStore.top(1)[0].lastSeen < THRESHOLD_TIME
        ) {
          peer = (this.node.network.peerStore.pop() as Entry).id
        } else {
          break
        }
      }

      tokens.push(token)
    }

    if (
      this.node.network.peerStore.peers.length > 0 &&
      this.node.network.peerStore.top(1)[0].lastSeen < THRESHOLD_TIME
    ) {
      let token = tokens.pop() as Token
      promises[token] = queryNode((this.node.network.peerStore.pop() as Entry).id, token)
    }

    await Promise.all(promises)
  }

  setTimeout() {
    this.timeout = setTimeout(async () => {
      await this.checkNodes()
      this.setTimeout()
    }, randomInteger(CHECK_INTERVAL_LOWER_BOUND, CHECK_INTERVAL_UPPER_BOUND))
  }

  start(): void {
    this.setTimeout()
    log(`Heartbeat mechanism started`)
  }

  stop(): void {
    clearTimeout(this.timeout)
    clearInterval(this.interval)
    log(`Heartbeat mechanism stopped`)
  }
}

export default Heartbeat
