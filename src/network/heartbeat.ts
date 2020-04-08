import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'

import PeerId from 'peer-id'
import type PeerInfo from 'peer-info'

import { EventEmitter } from 'events'

const TWO_MINUTES = 2 * 60 * 1000
const FORTY_ONE_SECONDS = 41 * 1000

const REFRESH_TIME = TWO_MINUTES
const CHECK_INTERVAL = FORTY_ONE_SECONDS

const MAX_PARALLEL_CONNECTIONS = 10

type PeerIdString = string
type PeerIdLastSeen = number

class Heartbeat<Chain extends HoprCoreConnector> extends EventEmitter {
  heap: string[] = []

  nodes = new Map<PeerIdString, PeerIdLastSeen>()

  interval: any

  constructor(public node: Hopr<Chain>) {
    super()

    this.node.on('peer:connect', (peerInfo: PeerInfo) => this.emit('beat', peerInfo.id))

    super.on('beat', this.connectionListener)
  }

  private connectionListener(peer: PeerId) {
    const peerIdString = peer.toB58String()

    let found = this.nodes.get(peerIdString)

    if (found == undefined) {
      this.heap.push(peerIdString)
    }

    this.nodes.set(peerIdString, Date.now())
  }

  private comparator(a: PeerIdString, b: PeerIdString) {
    let lastSeenA = this.nodes.get(a)
    let lastSeenB = this.nodes.get(b)

    if (lastSeenA == lastSeenB) {
      return 0
    }
    if (lastSeenA == undefined) {
      return 1
    }

    if (lastSeenB == undefined) {
      return -1
    }

    return lastSeenA < lastSeenB ? -1 : 1
  }

  async checkNodes() {
    const promises: Promise<void>[] = []

    this.heap = this.heap.sort(this.comparator.bind(this))

    const THRESHOLD_TIME = Date.now() - REFRESH_TIME

    // Remove non-existing nodes
    let index = this.heap.length - 1
    while (this.nodes.get(this.heap[index--]) == undefined) {
      this.heap.pop()
    }

    let heapIndex = 0

    const updateHeapIndex = (): void => {
      while (heapIndex < this.heap.length) {
        const lastSeen = this.nodes.get(this.heap[heapIndex])

        if (lastSeen == undefined || lastSeen > THRESHOLD_TIME) {
            heapIndex++
          continue
        } else {
          break
        }
      }
    }

    const queryNode = async (startIndex: number): Promise<void> => {
        let currentPeerId: PeerId
      while (startIndex < this.heap.length) {
        currentPeerId = PeerId.createFromB58String(this.heap[startIndex])
        try {
          await this.node.interactions.network.heartbeat.interact(currentPeerId)
          this.nodes.set(this.heap[startIndex], Date.now())
        } catch (err) {
          this.nodes.delete(this.heap[startIndex])
          this.node.peerStore.remove(currentPeerId)
        }

        startIndex = heapIndex
      }
    }

    updateHeapIndex()

    while (promises.length < MAX_PARALLEL_CONNECTIONS && heapIndex < this.heap.length) {
      promises.push(queryNode(heapIndex++))
    }

    await Promise.all(promises)
  }

  start(): void {
    this.interval = setInterval(this.checkNodes.bind(this), CHECK_INTERVAL)
  }

  stop(): void {
    clearInterval(this.interval)
  }
}

export { Heartbeat }
