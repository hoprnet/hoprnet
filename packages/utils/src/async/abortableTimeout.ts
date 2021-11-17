import { defer } from './defer'
import { debug } from '../process'

const logError = debug('hopr:lateTimeout')

export type TimeoutOpts = {
  timeout: number
  signal?: AbortSignal
}

export async function abortableTimeout<Result, AbortMsg, TimeoutMsg>(
  fn: (opts: TimeoutOpts) => Promise<Result>,
  abortMsg: AbortMsg,
  timeoutMsg: TimeoutMsg,
  opts: TimeoutOpts
): Promise<Result | AbortMsg | TimeoutMsg> {
  const abort = new AbortController()

  // forward abort request
  const onOuterAbort = () => abort.abort()
  opts.signal?.addEventListener('abort', () => {
    onOuterAbort()
    opts.signal?.removeEventListener('abort', onOuterAbort)
  })

  let done = false

  let onAbort: () => void
  let cleanUp: () => void

  const abortableTimeout = defer<AbortMsg | TimeoutMsg>()

  const timeout = setTimeout(() => {
    done = true
    cleanUp()
    abort.abort()
    abortableTimeout.resolve(timeoutMsg)
  }, opts.timeout)

  onAbort = () => {
    done = true
    cleanUp()
    abortableTimeout.resolve(abortMsg)
  }

  cleanUp = () => {
    clearTimeout(timeout)
    abort.signal.removeEventListener('abort', onAbort)
  }

  abort.signal.addEventListener('abort', onAbort)

  const resultFunction = async (): Promise<Result> => {
    try {
      const result = await fn({
        timeout: opts.timeout,
        signal: abort.signal
      })
      cleanUp()
      return result
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
      }
    }

    return
  }

  return Promise.race([abortableTimeout.promise, resultFunction()])
}
