import AbortController from 'abort-controller'

// Reject the body promise if it hasn't settled after timeout
export function timeoutAfter<T>(
  body: (abortSignal) => Promise<T>,
  timeout: number
): Promise<T> {
  const abortController = new AbortController()
  const timeoutPromise = new Promise<T>((_resolve, reject) => {
    setTimeout(() => {
      abortController.abort()
      reject('timeout exceeded')
    }, timeout)
  })
  const bodyPromise = body(abortController.signal)
  return Promise.race([
    timeoutPromise,
    bodyPromise
  ])
}
