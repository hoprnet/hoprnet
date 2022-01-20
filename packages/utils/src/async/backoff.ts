import { durations } from '../time'
import { debug } from '../process'
import { setTimeout } from 'timers/promises'

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
  } = {}
): Promise<T> {
  const { minDelay = durations.seconds(1), maxDelay = durations.minutes(10), delayMultiple = 2 } = options
  let delay = minDelay

  if (minDelay >= maxDelay) {
    throw Error('minDelay should be smaller than maxDelay')
  } else if (delayMultiple <= 1) {
    throw Error('delayMultiple should be larger than 1')
  }

  while (true) {
    try {
      return await fn()
    } catch (err) {
      if (delay >= maxDelay) {
        throw err
      }
      log(`failed, attempting again in ${delay} (${err})`)
    }

    await wait(delay)
    delay = delay * delayMultiple
  }
}
