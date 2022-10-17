import HeapPackage from 'heap-js'
// Issues with ESM;
// Code and types are loaded differently
import type { Heap as HeapType } from 'heap-js'

import { randomInteger } from '@hoprnet/hopr-utils'
import type { Packet } from './messages/index.js'
import { MAX_PACKET_DELAY } from './constants.js'

// @ts-ignore untyped package
import retimer from 'retimer'

//import debug from 'debug'
//const log = debug('hopr-core:mixer')

type MixerEntry = [number, Packet]

const comparator = (a: MixerEntry, b: MixerEntry): number => {
  return a[0] - b[0]
}

const { Heap } = HeapPackage

enum Result {
  Packet,
  End
}

/**
 * Mix packets.
 *
 * Currently an MVP version, that simply adds a random interval to their
 * priority.
 */
export class Mixer {
  protected queue: HeapType<MixerEntry>

  protected timer: any | undefined
  private nextPacket: (msg: Result) => void
  private nextPromise: Promise<Result>

  constructor(private nextRandomInt: (start: number, end: number) => number = randomInteger) {
    this.queue = new Heap(comparator)
    this.nextPromise = new Promise<Result>((resolve) => {
      this.nextPacket = resolve
    })

    this.push = this.push.bind(this)
    this.end = this.end.bind(this)
  }

  async *[Symbol.asyncIterator](): AsyncIterableIterator<Packet> {
    while (true) {
      const result = await this.nextPromise

      switch (result) {
        case Result.Packet:
          const latest = this.queue.pop()[1]

          this.nextPromise = new Promise((resolve) => {
            this.nextPacket = resolve
          })

          if (this.queue.length > 0) {
            const nextPriority = this.queue.top()[0][0]

            this.timer = retimer(() => this.nextPacket(Result.Packet), Math.max(nextPriority - Date.now(), 0))
          } else {
            this.timer = undefined
          }

          yield latest
          break
        case Result.End:
          return
      }
    }
  }

  public push(packet: Packet) {
    const packetPriority = this.getPriority()
    this.queue.push([packetPriority, packet])

    if (this.timer == undefined || this.queue.length == 1) {
      this.timer = retimer(() => {
        this.nextPacket(Result.Packet)
      }, Math.max(packetPriority - Date.now(), 0))

      return
    }

    const mostRecentPriority = this.queue.top(1)[0][0]

    if (packetPriority < mostRecentPriority) {
      this.timer.reschedule(Math.max(packetPriority - Date.now(), 0))
    }
  }

  public end() {
    this.timer?.reschedule(1)
    this.nextPacket(Result.End)
  }

  private getPriority(): number {
    return Date.now() + this.nextRandomInt(1, MAX_PACKET_DELAY)
  }

  public get pending(): number {
    return this.queue.length
  }
}
