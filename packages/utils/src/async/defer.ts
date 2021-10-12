export type DeferType<T> = {
  promise: Promise<T>
  resolve: (value?: T | PromiseLike<T>) => void
  reject: (reason?: any) => void
}

// Typed version of https://github.com/sindresorhus/p-defer
export function defer<T>(): DeferType<T> {
  const deferred = {} as DeferType<T>

  deferred.promise = new Promise<T>((resolve, reject) => {
    deferred.resolve = resolve
    deferred.reject = reject
  })

  return deferred
}
