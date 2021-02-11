import { Packet } from './messages/packet'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import Heap from 'heap-js'
import { randomInteger } from '@hoprnet/hopr-utils'
import { MAX_PACKET_DELAY } from './constants'
import Defer from 'p-defer'
import type { DeferredPromise } from 'p-defer'

import Debug from 'debug'
const log = Debug('hopr-core:mixer')

type HeapElement = [number, Packet<any>]

const comparator = (a: HeapElement, b: HeapElement): number => a[0] - b[0]

/**
 * Mix packets.
 *
 * Currently an MVP version, that simply adds a random interval to their
 * priority.
 */
export class Mixer<Chain extends HoprCoreConnector> {
  private queue: Heap<HeapElement>
  private timeout?: NodeJS.Timeout
  private poppable: DeferredPromise<void>
  private deferEnd: DeferredPromise<void>
  private endPromise: Promise<void>
  private _done: boolean

  public WAIT_TIME = MAX_PACKET_DELAY

  constructor(private incrementer = Date.now) {
    this.queue = new Heap(comparator)

    this.poppable = Defer<void>()
    this.deferEnd = Defer<void>()

    this._done = false

    this.endPromise = this.deferEnd.promise.then(() => {
      this._done = true
    })
  }

  /**
   * Adds a packet to the mixer
   * @param p
   */
  public push(p: Packet<Chain>) {
    if (this._done) {
      throw Error(`Mixer has ended. Could not accept any further messages.`)
    }
    const newPriority = this.getPriority()
    const topPriority = this.queue.length > 0 ? this.queue.peek()[0] : Number.MAX_SAFE_INTEGER

    if (newPriority < topPriority) {
      this.resetTimeout(newPriority)
    }

    log(`Added 1 packet to the mixer`)
    this.queue.push([newPriority, p])
  }

  /**
   * Get the number of packets that are currently
   * in the mixer
   */
  public get length() {
    return this.queue.length
  }

  /**
   * Adjust the internal timeout with latest priority
   * @param newPriority
   */
  private resetTimeout(newPriority: number) {
    clearTimeout(this.timeout)
    this.timeout = setTimeout(() => {
      this.poppable.resolve()
    }, newPriority - Date.now())
  }

  notEmpty(): boolean {
    return this.queue.length > 0
  }

  /**
   * Stops the mixer.
   */
  public stop(): void {
    log(`Ending mixer. Mixer will not accept any further messages`)

    if (this._done) {
      return
    }

    this.deferEnd.resolve()
  }

  public get done() {
    return this._done
  }

  private getPriority(): number {
    // @TODO implement fancy distribution sampling
    return this.incrementer() + randomInteger(1, MAX_PACKET_DELAY)
  }

  async pop(): Promise<undefined | Packet<Chain>> {
    await Promise.race([
      // prettier-ignore
      this.endPromise,
      this.poppable.promise
    ])

    // return once done
    if (this._done) {
      return undefined
    }

    const result = this.queue.pop()[1]

    // reset promise to wait for next message
    this.poppable = Defer<void>()

    log(`Removed 1 packet from mixer. ${this.queue.length} packet are waiting.`)

    // reset timeout only if there is another
    // message, otherwise wait for the next .push()
    if (!this.queue.isEmpty()) {
      this.resetTimeout(this.queue.peek()[0])
    }

    return result
  }
}
