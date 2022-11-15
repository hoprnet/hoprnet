export type DeferType<T> = {
  promise: Promise<T>
  resolve: (value: T | PromiseLike<T>) => void
  reject: (reason?: any) => void
}

// Typed version of https://github.com/sindresorhus/p-defer
export function defer<T>(): DeferType<T> {
  let resolve: (arg: T) => void
  let reject: (err: any) => void

  const promise = new Promise<T>((innerResolve, innerReject) => {
    resolve = innerResolve
    reject = innerReject
  })

  return {
    promise,
    resolve,
    reject
  }
}
