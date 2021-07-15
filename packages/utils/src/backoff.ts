import { durations } from './time'

export async function wait(milliseconds: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, milliseconds))
}

/**
 * A general use backoff that will reject once MAX_DELAY is reached.
 * @param fn asynchronous function to run on every tick
 * @param options.minDelay minimum delay, we start with this
 * @param options.maxDelay maximum delay, we reject once we reach this
 * @param options.delayMultiple multiplier to apply to increase running delay
 */
export async function backoff(
  fn: () => Promise<any>,
  options: {
    minDelay?: number
    maxDelay?: number
    delayMultiple?: number
  } = {}
): ReturnType<typeof fn> {
  const { minDelay = durations.seconds(1), maxDelay = durations.minutes(10), delayMultiple = 2 } = options
  let delay: number | undefined

  if (minDelay >= maxDelay) throw Error('minDelay should be smaller than maxDelay')
  else if (delayMultiple < 1) throw Error('delayMultiple should be larger than 1')

  return new Promise<ReturnType<typeof fn>>(async (resolve, reject) => {
    const tick = async () => {
      try {
        const result = await fn()
        return resolve(result)
      } catch (err) {
        if (delay === maxDelay) {
          return reject(err)
        }

        // if delay is not set, our first delay is minDelay
        // else we start exp increasing
        delay = typeof delay === 'undefined' ? minDelay : Math.min(delay * delayMultiple, maxDelay)

        await wait(delay)
        return tick()
      }
    }

    return tick()
  })
}
