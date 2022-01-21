import { FIFO } from '../collection'
import { debug } from '../process'

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

  function push(fn: () => Promise<ReturnType>): void {
    queue.push(fn)

    if (queue.size() == 1) {
      start()
    }
  }

  async function start(): Promise<void> {
    while (queue.size() > 0) {
      try {
        await queue.shift()()
      } catch (err) {
        log(err)
      }
    }
  }

  return push
}
