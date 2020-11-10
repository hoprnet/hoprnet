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

  public WAIT_TIME = MAX_PACKET_DELAY

  constructor(private incrementer = Date.now) {
    this.queue = new Heap(comparator)
  }

  push(p: Packet<Chain>) {
    this.queue.push([this.getPriority(), p])
  }

  // Can we pop an element?.
  poppable(): boolean {
    log(`Mixer has ${this.queue.length} elements`)
    if (!this.queue.length) {
      return false
    }
    return this.queue.peek()[0] < this.due()
  }

  pop(): Packet<Chain> {
    if (!this.poppable()) {
      throw new Error('No packet is ready to be popped from mixer')
    }
    return this.queue.pop()[1]
  }

  notEmpty(): boolean {
    return this.queue.length > 0
  }

  private due(): number {
    return this.incrementer()
  }

  private getPriority(): number {
    return this.incrementer() + randomInteger(1, MAX_PACKET_DELAY)
  }
}
