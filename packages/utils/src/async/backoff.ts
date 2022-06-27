import { durations } from '../time.js'
import { debug } from '../process/index.js'
import { setTimeout, setImmediate } from 'timers/promises'

const log = debug('hopr:utils:retry')

export async function wait(milliseconds: number): Promise<void> {
  await setTimeout(milliseconds)
}

/**
 * A general use backoff that will reject once MAX_DELAY is reached.
 * @param fn asynchronous function to run on every tick
 * @param options.minDelay minimum delay, we start with this
 * @param options.maxDelay maximum delay, we reject once we reach this
 * @param options.delayMultiple multiplier to apply to increase running delay
 */
export async function retryWithBackoff<T>(
  fn: () => Promise<T>,
  options: {
    minDelay?: number
    maxDelay?: number
    delayMultiple?: number
  } = { minDelay: durations.seconds(1), maxDelay: durations.minutes(10), delayMultiple: 2 }
): Promise<T> {
  let delay = options.minDelay

  if (options.minDelay >= options.maxDelay) {
    throw Error('minDelay should be smaller than maxDelay')
  } else if (options.delayMultiple <= 1) {
    throw Error('delayMultiple should be larger than 1')
  }

  while (true) {
    try {
      return await fn()
    } catch (err) {
      if (delay >= options.maxDelay) {
        throw err
      }
      log(`failed, attempting again in ${delay} (${err})`)
    }

    await setTimeout(delay)

    // Give other tasks CPU time to happen
    // Push next loop iteration to end of next event loop iteration
    await setImmediate()

    delay = delay * options.delayMultiple
  }
}
