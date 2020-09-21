import { durations } from '../time'

/**
 * Starts an interval after a timeout.
 *
 * @param msg message to display
 */
export function startDelayedInterval(msg: string): () => void {
  let interval: NodeJS.Timeout | number
  let timeout = setTimeout(() => {
    process.stdout.write(`<${msg}>\n`)
    interval = setInterval(() => {
      process.stdout.write('.')
    }, durations.seconds(1))
  }, durations.seconds(2))

  return function dispose() {
    clearTimeout(timeout)
    clearInterval(interval as number)
  }
}
