import { defer } from './defer'
import { debug } from '../process'

const logError = debug('hopr:lateTimeout')

export type TimeoutOpts = {
  timeout: number
  signal?: AbortSignal
}

export async function abortableTimeout<Result, AbortMsg, TimeoutMsg>(
  fn: (opts: Required<TimeoutOpts>) => Promise<Result>,
  abortMsg: AbortMsg,
  timeoutMsg: TimeoutMsg,
  opts: TimeoutOpts
): Promise<Result | AbortMsg | TimeoutMsg> {
  const abort = new AbortController()

  let done = false

  const onceDone = defer<AbortMsg | TimeoutMsg>()

  const cleanUp = () => {
    done = true
    abort.signal.removeEventListener('abort', onAbort)
  }

  const onTimeout = () => {
    if (done) {
      return
    }
    cleanUp()
    abort.abort()
    onceDone.resolve(timeoutMsg)
  }

  const onAbort = () => {
    if (done) {
      return
    }
    cleanUp()
    onceDone.resolve(abortMsg)
  }

  abort.signal.addEventListener('abort', onAbort)

  const resultFunction = async (): Promise<Result> => {
    let result: Result
    try {
      result = await fn({
        timeout: opts.timeout,
        signal: abort.signal
      })
    } catch (err) {
      if (!done) {
        throw err
      } else {
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

    cleanUp()
    return result
  }

  // forward abort request
  const onOuterAbort = abort.abort.bind(abort)
  opts.signal?.addEventListener('abort', () => {
    onOuterAbort()
    opts.signal?.removeEventListener('abort', onOuterAbort)
  })

  // Let the timeout run through and let the handler do nothing, i.e. `done = true`
  // instead of clearing the timeout with `clearTimeout` which becomes an expensive
  // operation when using many timeouts
  setTimeout(onTimeout, opts.timeout)

  return Promise.race([onceDone.promise, resultFunction()])
}
