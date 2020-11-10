import { Packet } from './messages/packet'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import Heap from 'heap-js'
import { randomInteger } from '@hoprnet/hopr-utils'
import { MAX_PACKET_DELAY } from './constants'
import debug from 'debug'
const log = debug('hopr-core:mixer')

type HeapElement = [number, Packet<any>]

let comparator = (a: HeapElement, b: HeapElement): number => {
  if (b[0] > a[0]) {
    return 1
  } else if (b[0] < a[0]) {
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

  constructor() {
    this.queue = new Heap(comparator)
  }

  push(p: Packet<Chain>) {
    this.queue.push([this.getPriority(), p])
  }

  // Can we pop an element?.
  poppable(): boolean {
    log(`Mixer has ${this.queue.length} elements`)
    return this.queue.peek()[0] > this.due()
  }

  pop(): Packet<Chain> {
    let elem = this.queue.pop()
    if (elem[0] <= this.due()) {
      throw new Error('No packet is ready to be popped from mixer')
    }
    return elem[1]
  }

  private due(): number {
    return Date.now()
  }

  private getPriority(): number {
    return Date.now() + randomInteger(0, MAX_PACKET_DELAY)
  }
}
