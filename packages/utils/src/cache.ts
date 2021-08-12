import { isExpired } from '.'

// Cache the result of a function until expiry, and return that.
//
// ie.
//
// let cachedFunction = cacheNoArgAsyncFunction<ReturnType>(expensiveAsyncFunction, 500)
export function cacheNoArgAsyncFunction<T>(func: () => Promise<T>, expiry: number) {
  let cachedValue: T
  let updatedAt: number
  return async function (): Promise<T> {
    if (cachedValue && !isExpired(updatedAt, new Date().getTime(), expiry)) {
      return cachedValue
    }
    cachedValue = await func()
    updatedAt = new Date().getTime()
    return cachedValue
  }
}
