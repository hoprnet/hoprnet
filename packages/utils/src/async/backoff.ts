import { durations } from '../time.js'
import { debug } from '../process/index.js'
import { setTimeout } from 'timers/promises'

const log = debug('hopr:utils:retry')

// Throws after ~170 minutes (2 hours 50 minutes, 50 seconds)
// and 10 retries if function has not been successful
export const DEFAULT_BACKOFF_PARAMETERS = {
  minDelay: durations.seconds(1),
  maxDelay: durations.minutes(10),
  delayMultiple: 2
}

export async function wait(milliseconds: number): Promise<void> {
  await setTimeout(milliseconds)
}

/**
 * Returns the maximal number of retries after which the `retryWithBackoff` throws
 * @param minDelay initial delay
 * @param maxDelay maximal delay to retry
 * @param delayMultiple factor by which last delay got multiplied
 * @returns
 */
export function getBackoffRetries(minDelay: number, maxDelay: number, delayMultiple: number) {
  //
  //     ┌─                      ─┐
  //     │        maxDelay        │
  //     │ log_2( ───────────── ) │
  //     │        delayMultiple   │
  // n = │ ────────────────────── │
  //     │ log_2( delayMultiple ) │
  //     │                        │
  //
  return Math.ceil(Math.log2(maxDelay / minDelay) / Math.log2(delayMultiple))
}

/**
 * Returns the *total* amount of time between calling `retryWithBackThenThrow` and
 * once it throws because it ran out of retries.
 *
 * @param minDelay initial delay
 * @param maxDelay maximal delay to retry
 * @param delayMultiple factor by which last delay got multiplied
 * @returns
 */
export function getBackoffRetryTimeout(minDelay: number, maxDelay: number, delayMultiple: number) {
  const retries = getBackoffRetries(minDelay, maxDelay, delayMultiple) - 1

  if (retries < 0) {
    // `retryWithBackThenThrow` throws after first invocation
    return 0
  }

  if (delayMultiple == 1) {
    throw Error(`boom`)
    return minDelay * (retries + 1)
  } else {
    // n-th partial sum of geometric series
    // see https://en.wikipedia.org/wiki/Geometric_series#Sum
    //
    //                                retries + 1
    //                    delayMultple            - 1
    // s_n = minDelay * ( ─────────────────────────── )
    //                         delayMultiple - 1
    //
    return minDelay * ((delayMultiple ** (retries + 1) - 1) / (delayMultiple - 1))
  }
}

/**
 * A general-use exponential backoff that will throw once
 * iteratively increased timeout reaches MAX_DELAY.
 *
 * @dev this function THROWS if retries were not successful
 *
 * @param fn asynchronous function to run on every tick
 * @param options.minDelay minimum delay, we start with this
 * @param options.maxDelay maximum delay, we reject once we reach this
 * @param options.delayMultiple multiplier to apply to increase running delay
 */
export async function retryWithBackoffThenThrow<T>(
  fn: () => Promise<T>,
  options: {
    minDelay?: number
    maxDelay?: number
    delayMultiple?: number
  } = DEFAULT_BACKOFF_PARAMETERS
): Promise<T> {
  if (options.minDelay >= options.maxDelay) {
    throw Error('minDelay should be smaller than maxDelay')
  } else if (options.delayMultiple <= 1) {
    throw Error('delayMultiple should be larger than 1')
  }

  let delay = options.minDelay

  const retryIterator = (async function* () {
    while (true) {
      try {
        yield await fn()
        break
      } catch (err) {
        if (delay >= options.maxDelay) {
          throw err
        }
        log(`failed, attempting again in ${delay} (${err})`)
      }

      await setTimeout(delay)

      // Node.JS can reschedule iteration at any point in time
      yield

      delay = delay * options.delayMultiple
    }
  })()

  for await (const result of retryIterator) {
    if (result) {
      return result
    }
  }
}
