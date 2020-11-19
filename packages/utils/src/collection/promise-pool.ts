export async function limitConcurrency<T>(
  maxConcurrency: number,
  exitCond: () => boolean,
  createPromise: () => Promise<T>
): Promise<T[]> {
  const ret: Promise<T>[] = []
  const executing = []
  while (!exitCond()) {
    const p = createPromise()
    ret.push(p)
    const e = p.then(() => executing.splice(executing.indexOf(e), 1))
    executing.push(e)
    if (executing.length >= maxConcurrency) {
      await Promise.race(executing)
    }
  }
  return Promise.all(ret)
}
