import { durations } from './time'

export async function wait(milliseconds: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, milliseconds))
}

export async function backoff(
  fn: () => Promise<any>,
  options: {
    minDelay?: number
    maxDelay?: number
    delayMultiple?: number
  } = {}
): ReturnType<typeof fn> {
  const { minDelay = durations.seconds(1), maxDelay = durations.minutes(10), delayMultiple = 2 } = options
  let delay = minDelay

  return new Promise<ReturnType<typeof fn>>(async (resolve, reject) => {
    const tick = async () => {
      try {
        const result = await fn()
        return resolve(result)
      } catch (err) {
        delay = delay * delayMultiple

        if (delay > maxDelay) {
          return reject(err)
        }

        await wait(delay)
        return tick()
      }
    }

    return tick()
  })
}
