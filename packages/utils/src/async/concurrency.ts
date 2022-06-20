import { FIFO } from '../collection/index.js'
import { debug } from '../process/index.js'

const log = debug('hopr:concurrency-limitter')
/**
 * Creates a limiter that takes functions and runs them subsequently
 * with no concurrency.
 * @example
 * let limiter = oneAtATime()
 * limiter(() => Promise.resolve('1'))
 * limiter(() => Promise.resolve('2'))
 * @returns a limiter that takes additional functions
 */
export function oneAtATime<ReturnType>(): (fn: () => Promise<ReturnType>) => void {
  const queue = FIFO<() => Promise<ReturnType>>()
  let isRunning: boolean = false

  function push(fn: () => Promise<ReturnType>): void {
    queue.push(fn)
    maybeStart()
  }

  function maybeStart(): void {
    if (queue.size() == 1 && !isRunning) {
      start()
    }
  }

  async function start(): Promise<void> {
    isRunning = true
    while (queue.size() > 0) {
      try {
        await queue.shift()()
      } catch (err) {
        log(err)
      }
    }
    isRunning = false
  }

  return push
}
