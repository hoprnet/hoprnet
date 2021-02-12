import { Packet } from './messages/packet'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import Heap from 'heap-js'
import { randomInteger } from '@hoprnet/hopr-utils'
import { MAX_PACKET_DELAY } from './constants'
import debug from 'debug'
const log = debug('hopr-core:mixer')

type HeapElement = [number, Packet<any>]

const comparator = (a: HeapElement, b: HeapElement): number => {
  if (b[0] < a[0]) {
    return 1
  } else if (b[0] > a[0]) {
    return -1
  }
  return 0
}

/**
 * Mix packets.
 *
 * Currently an MVP version, that simply adds a random interval to their
 * priority.
 */
export class Mixer<Chain extends HoprCoreConnector> {
  private queue: Heap<HeapElement>
  private next: NodeJS.Timeout

  public WAIT_TIME = MAX_PACKET_DELAY

  constructor(private onMessage: (m: Packet<Chain>) => void, private clock = Date.now) {
    this.queue = new Heap(comparator)
  }

  public push(p: Packet<Chain>) {
    this.queue.push([this.getPriority(), p])
    this.addTimeout()
  }

  private addTimeout() {
    if (!this.next && this.queue.length > 0){
      this.next = setTimeout(this.tick.bind(this), this.intervalUntilNextMessage())
    }
  }

  private tick(){
    log(`Mixer has ${this.queue.length} elements`)
    while (this.queue.length > 0 && this.queue.peek()[0] < this.clock()){
      this.onMessage(this.queue.pop()[1])
    }
    this.next = null
    this.addTimeout()
  }

  private intervalUntilNextMessage(): number {
    return Math.max(this.queue.peek()[0] - this.clock(), 1)
  }

  private getPriority(): number {
    return this.clock() + randomInteger(1, MAX_PACKET_DELAY)
  }
}
