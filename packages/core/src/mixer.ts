import { Packet } from './messages/packet'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import Heap from 'heap-js'
import { randomInteger } from '@hoprnet/hopr-utils'
import { MAX_PACKET_DELAY } from './constants'
import Defer from 'p-defer'
import type { DeferredPromise } from 'p-defer'

// @TODO add logging

type HeapElement = [number, Packet<any>]

const comparator = (a: HeapElement, b: HeapElement): number => {
  return a[0] - b[0]
}

/**
 * Mix packets.
 *
 * Currently an MVP version, that simply adds a random interval to their
 * priority.
 */
export class Mixer<Chain extends HoprCoreConnector> {
  private queue: Heap<HeapElement>
  private timeout?: NodeJS.Timeout
  private defer: DeferredPromise<void>
  private deferEnd: DeferredPromise<void>
  private endPromise: Promise<void>
  private _done: boolean

  public WAIT_TIME = MAX_PACKET_DELAY

  constructor(private incrementer = Date.now) {
    this.queue = new Heap(comparator)

    this.defer = Defer<void>()
    this.deferEnd = Defer<void>()

    this._done = false

    this.endPromise = this.deferEnd.promise.then(() => {
      this._done = true
    })
  }

  public push(p: Packet<Chain>) {
    const newPriority = this.getPriority()
    const topPriority = this.queue.length > 0 ? this.queue.peek()[0] : Number.MAX_SAFE_INTEGER

    if (newPriority < topPriority) {
      this.resetTimeout(newPriority)
    }

    this.queue.push([newPriority, p])
  }

  public get length() {
    return this.queue.length
  }

  private resetTimeout(newPriority: number) {
    clearTimeout(this.timeout)
    this.timeout = setTimeout(() => {
      this.defer.resolve()
    }, newPriority - Date.now())
  }

  notEmpty(): boolean {
    return this.queue.length > 0
  }

  public end(): void {
    this.deferEnd.resolve()
  }

  public get done() {
    return this._done
  }

  private getPriority(): number {
    return this.incrementer() + randomInteger(1, MAX_PACKET_DELAY)
  }

  async *[Symbol.asyncIterator](): AsyncGenerator<Packet<Chain>> {
    while (true) {
      await Promise.race([
        // prettier-ignore
        this.endPromise,
        this.defer.promise
      ])

      if (this._done) {
        break
      }
      this.defer = Defer<void>()

      const result = this.queue.pop()[1]

      if (!this.queue.isEmpty()) {
        this.resetTimeout(this.queue.peek()[0])
      }

      yield result
    }
  }
}
