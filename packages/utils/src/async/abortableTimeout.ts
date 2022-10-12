import { debug } from '../process/index.js'

const logError = debug('hopr:lateTimeout')

export type TimeoutOpts = {
  timeout: number
  signal?: AbortSignal
}

/**
 * Cals the worker function with a timeout. Once the timeout is done
 * abort the call using an abort controller.
 * If the caller aims to end the call before the tiemout has happened
 * it can pass an AbortController and end the call prematurely.
 * @param fn worker function to dial
 * @param abortMsg value to be returned if aborted
 * @param timeoutMsg value to be returned on timeout
 * @param opts options to pass to worker function
 * @returns
 */
export function abortableTimeout<Result, AbortMsg, TimeoutMsg>(
  fn: (opts: Required<TimeoutOpts>) => Promise<Result>,
  abortMsg: AbortMsg,
  timeoutMsg: TimeoutMsg,
  opts: TimeoutOpts
): Promise<Result | AbortMsg | TimeoutMsg> {
  return new Promise<Result | AbortMsg | TimeoutMsg>(async (resolve, reject) => {
    const abort = new AbortController()

    let result: Result

    let done = false

    let timeout: NodeJS.Timeout
    // forward outer abort
    const innerAbort = abort.abort.bind(abort)

    const cleanUp = () => {
      clearTimeout(timeout)
      done = true
      abort.signal.removeEventListener('abort', onInnerAbort)
      opts.signal?.removeEventListener('abort', innerAbort)
    }

    const onTimeout = () => {
      if (done) {
        return
      }
      cleanUp()
      innerAbort()
      resolve(timeoutMsg)
    }

    const onInnerAbort = () => {
      if (done) {
        return
      }
      cleanUp()
      resolve(abortMsg)
    }

    abort.signal.addEventListener('abort', onInnerAbort)

    opts.signal?.addEventListener('abort', innerAbort)

    // Let the timeout run through and let the handler do nothing, i.e. `done = true`
    // instead of clearing the timeout with `clearTimeout` which becomes an expensive
    // operation when using many timeouts
    timeout = setTimeout(onTimeout, opts.timeout)

    try {
      result = await fn({
        timeout: opts.timeout,
        signal: abort.signal
      })
    } catch (err) {
      if (!done) {
        cleanUp()
        reject(err)
        return
      } else {
        cleanUp()
        // The function has thrown an error after there
        // was a timeout or the call has been aborted.
        // The abortableTimeout has returned a value, hence
        // the caller is no longer interested in errors.
        // Nevertheless, in order to debug these errors and
        // to prevent from uncaught promise rejection, the error
        // are logged.
        logError(`Function has thrown an error after the timeout happend or the function got aborted`, err)
        return
      }
    }

    if (done) {
      return
    }

    cleanUp()

    resolve(result)
  })
}
