export async function limitConcurrency<T>(
  maxConcurrency: number,
  exitCond: () => boolean,
  createPromise: () => Promise<T>,
  maxIterations: number = 1e3
): Promise<T[]> {
  const ret: Promise<T>[] = []
  const executing: Promise<void>[] = []
  let i = 0
  while (!exitCond() && i++ < maxIterations) {
    const p = createPromise()
    ret.push(p)
    const e: Promise<void> = p.then(() => { executing.splice(executing.indexOf(e), 1) })
    executing.push(e)
    if (executing.length >= maxConcurrency) {
      await Promise.race(executing)
    }
  }
  return Promise.all(ret)
}
