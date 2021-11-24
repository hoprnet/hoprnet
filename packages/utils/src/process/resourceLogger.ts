import { memoryUsage, resourceUsage } from 'process'

type LogType = (msg: string) => void | Promise<void>

function createResourceLog(log: LogType) {
  const resourcesUsed = resourceUsage()
  // reported as KiloBytes
  const maxUsedMemory = resourcesUsed.maxRSS / 1024
  // reported in microseconds
  const usedCpu = resourcesUsed.userCPUTime / 1000 / 1000
  // reported as Bytes
  const usedMemory = memoryUsage().rss / 1024 / 1024

  log(`Process stats: mem ${usedMemory.toPrecision(1)} MB (max: ${maxUsedMemory.toPrecision(1)} MB) ` + `cputime: ${usedCpu.toPrecision(1)} sec`)
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
