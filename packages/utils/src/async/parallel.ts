/**
 * Runs the asynchronous task and fetches synchronous and asynchronously thrown errors
 * and performs the callback at the end of the next iteration of the event loop.
 * @param fn worker function
 * @param arg a single argument to pass to the worker function
 * @param resultIndex index in the results array
 * @returns a decorated worker result
 */
async function runTask<ArgType, Return, Args extends Array<ArgType>>(
  fn: (...args: Args) => Promise<Return>,
  arg: Args,
  resultIndex: number,
  update: (resultIndex: number, result: Return | Error) => void
): Promise<void> {
  try {
    const value = await fn(...arg)
    setImmediate(update, resultIndex, value)
  } catch (err) {
    setImmediate(update, resultIndex, err)
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
export function nAtATime<ArgType, Return, Args extends Array<ArgType>>(
  fn: (...args: Args) => Promise<Return>,
  args: Iterable<Args>,
  concurrency: number,
  done?: (results: (Return | Error | undefined)[]) => boolean
): Promise<(Return | Error)[]> {
  const it = args[Symbol.iterator]()

  let chunk = it.next()
  if (concurrency <= 0 || chunk.done) {
    return Promise.resolve([])
  }

  return new Promise<(Return | Error)[]>((resolve) => {
    const results = []

    let currentIndex = 0
    let activeWorkers = 0
    let ending = false

    const update = (resultIndex: number, result: Return | Error) => {
      // console.log(
      //   `updating: resultIndex ${resultIndex} currentIndex ${currentIndex} activeWorkers ${activeWorkers}`,
      //   results
      // )
      results[resultIndex] = result

      if (done != undefined) {
        ending = ending || done(results)
      }

      if (!ending && !chunk.done) {
        // Create an array entry for the result
        results.push()
        runTask(fn, chunk.value, currentIndex, update)
        chunk = it.next()
        currentIndex++
      } else {
        if (activeWorkers == 1) {
          resolve(results)
        } else {
          activeWorkers--
        }
      }
    }

    while (!chunk.done && currentIndex < concurrency) {
      activeWorkers++
      // Create an array entry for the result
      results.push()
      runTask(fn, chunk.value, currentIndex, update)
      currentIndex++
      chunk = it.next()
    }
  })
}
