import { memoryUsage, resourceUsage } from 'process'

type LogType = (msg: string) => void | Promise<void>

function createResourceLog(log: LogType) {
  const used = memoryUsage()
  const usage = resourceUsage()
  log(`Process stats: mem ${used.rss / 1024}k (max: ${usage.maxRSS / 1024}k) ` + `cputime: ${usage.userCPUTime}`)
}

/**
 * Creates a resource logger and provides a function to stop it.
 * @param log logs resource stat strings
 * @param ms interval to redo the measurement
 * @returns a function that stop the resource logger
 */
export function startResourceUsageLogger(log: LogType, ms = 60_000): () => void {
  createResourceLog(log)

  const interval = setInterval(() => createResourceLog(log), ms)

  return () => clearInterval(interval)
}
