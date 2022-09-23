/**
 * Races a timeout against some work
 * @param timeout return after timeout in ms
 * @param work function that returns a Promise that resolves once the work is done
 * @returns a Promise that resolves once the timeout is due or the work is done
 */
export function timeout<T>(timeout: number, work: () => Promise<T>): Promise<T> {
  let resolve: any
  let reject: any

  let done = false

  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })

  const onReject = (err?: any) => {
    if (done) {
      return
    }
    done = true

    reject(err)
  }

  const onResolve = (res: T) => {
    if (done) {
      return
    }
    done = true

    resolve(res)
  }

  setTimeout(onReject, timeout, Error('Timeout'))

  try {
    work().then(onResolve, onReject)
  } catch (err) {
    onReject(err)
  }

  return promise
}
