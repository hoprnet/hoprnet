type WorkerType<Return> = Promise<{
  resultIndex: number
  workerIndex: number
  value: Return
}>

/**
 * Decorates the call of the worker function to return the
 * index in the array of workers.
 * @param fn worker function
 * @param arg a single argument to pass to the worker function
 * @param workerIndex index in the worker array
 * @param workerIndex index in the results array
 * @returns a decorated worker result
 */
async function decorateWorker<ArgType, Return, Args extends Array<ArgType>>(
  fn: (...args: Args) => Promise<Return>,
  arg: Args,
  workerIndex: number,
  resultIndex: number
): WorkerType<Return> {
  try {
    return { resultIndex, workerIndex, value: await fn(...arg) }
  } catch (err) {
    return { resultIndex, workerIndex, value: err }
  }
}

/**
 * Runs the same worker function with multiple arguments but does not run more
 * than a given number of workers concurrently.
 * @dev Iterative implementation of the functionality
 * @param fn worker function
 * @param args arguments passed to worker function
 * @param concurrency number of parallel jobs
 * @returns an array containing the results
 * @example
 * import { setTimeout } from 'timers/promises'
 *
 * const result = await nAtaTime(setTimeout, [[300, 'one'], [200, 'two'], [100, 'three']], 2)
 * // => ['two', 'one', 'three']
 */
export async function nAtATime<ArgType, Return, Args extends Array<ArgType>>(
  fn: (...args: Args) => Promise<Return>,
  args: Args[],
  concurrency: number
): Promise<Return[]> {
  if (concurrency <= 0) {
    return []
  }

  let currentIndex = Math.min(concurrency, args.length)

  const workers: (WorkerType<Return> | undefined)[] = Array.from({ length: currentIndex }, (_, index: number) =>
    decorateWorker(fn, args[index], index, index)
  )

  let activeWorkers = currentIndex

  const results = new Array<Return>(args.length)

  while (activeWorkers > 0) {
    let functionResult: Awaited<WorkerType<Return>>
    if (activeWorkers == concurrency) {
      functionResult = await Promise.race(workers)
    } else {
      functionResult = await Promise.race(workers.filter((worker: WorkerType<Return>) => worker))
    }

    results[functionResult.resultIndex] = functionResult.value

    if (currentIndex < args.length) {
      workers[functionResult.workerIndex] = decorateWorker(
        fn,
        args[currentIndex],
        functionResult.workerIndex,
        currentIndex
      )
      currentIndex++
    } else {
      workers[functionResult.workerIndex] = undefined
      activeWorkers--
    }
  }

  return results
}
