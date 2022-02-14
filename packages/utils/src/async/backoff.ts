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
export function retryWithBackoff<T>(
  fn: () => Promise<T>,
  options: {
    minDelay?: number
    maxDelay?: number
    delayMultiple?: number
  } = {}
): Promise<T> {
  const { minDelay = durations.seconds(1), maxDelay = durations.minutes(10), delayMultiple = 2 } = options

  if (minDelay >= maxDelay) {
    return Promise.reject(Error('minDelay should be smaller than maxDelay'))
  } else if (delayMultiple <= 1) {
    return Promise.reject(Error('delayMultiple should be larger than 1'))
  }

  return new Promise<T>((resolve, reject) => {
    // Use call-by-reference
    const state = {
      delay: minDelay
    }

    const fetch = () => {
      fn().then(resolve, (err: any) => {
        if (state.delay >= maxDelay) {
          reject(err)
        }
        log(`failed, attempting again in ${state.delay} (${err})`)

        setImmediate(() => {
          wait(state.delay).then(fetch)
          state.delay = state.delay * delayMultiple
        })
      })
    }

    fetch()
  })
}
