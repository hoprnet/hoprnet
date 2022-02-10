import type { TimeoutOpts } from './abortableTimeout'

import assert from 'assert'
import AbortController from 'abort-controller'
import { abortableTimeout } from './abortableTimeout'

enum Messages {
  ABORT_MSG,
  TIMEOUT_MSG,
  RESULT_MSG
}

describe('abortable timeout', function () {
  it('normal timeout behavior', async function () {
    const result = abortableTimeout(
      // Promise that does not resolve
      () => new Promise(() => {}) as any,
      Messages.ABORT_MSG,
      Messages.TIMEOUT_MSG,
      {
        timeout: 100
      }
    )

    assert((await result) === Messages.TIMEOUT_MSG)
  })

  it('normal abort behavior', async function () {
    const abort = new AbortController()

    const result = abortableTimeout(
      // Promise that does not resolve
      () => new Promise(() => {}) as any,
      Messages.ABORT_MSG,
      Messages.TIMEOUT_MSG,
      {
        timeout: 100,
        signal: abort.signal
      }
    )

    setTimeout(() => abort.abort(), 50)
    assert((await result) === Messages.ABORT_MSG)
  })

  it('normal result behavior', async function () {
    const result = abortableTimeout(
      // Promise that resolves with expected message
      (_opts: TimeoutOpts): Promise<Messages.RESULT_MSG> => Promise.resolve(Messages.RESULT_MSG),
      Messages.ABORT_MSG,
      Messages.TIMEOUT_MSG,
      {
        timeout: 100
      }
    )

    assert((await result) === Messages.RESULT_MSG)
  })

  it('timeout result behavior', async function () {
    let abortCalled = false

    const resultFunction = async (opts: TimeoutOpts): Promise<Messages.RESULT_MSG> => {
      return new Promise<Messages.RESULT_MSG>((resolve) => {
        opts.signal.addEventListener('abort', () => {
          abortCalled = true
        })

        setTimeout(() => {
          resolve(Messages.RESULT_MSG)
        }, 150)
      })
    }

    const result = abortableTimeout(resultFunction, Messages.ABORT_MSG, Messages.TIMEOUT_MSG, {
      timeout: 100
    })

    assert((await result) === Messages.TIMEOUT_MSG)

    assert(abortCalled)
  })

  it('forward abort call', async function () {
    let abortCalled = false
    const abort = new AbortController()

    const resultFunction = async (opts: TimeoutOpts): Promise<Messages.RESULT_MSG> =>
      new Promise<Messages.RESULT_MSG>((resolve) => {
        opts.signal.addEventListener('abort', () => {
          abortCalled = true
        })

        setTimeout(() => {
          resolve(Messages.RESULT_MSG)
        }, 100)
      })
    {
    }

    const result = abortableTimeout(resultFunction, Messages.ABORT_MSG, Messages.TIMEOUT_MSG, {
      timeout: 100,
      signal: abort.signal
    })

    setTimeout(() => abort.abort(), 50)

    assert((await result) === Messages.ABORT_MSG)

    assert(abortCalled)
  })

  it('timeout after result', async function () {
    let abortCalled = false
    const abort = new AbortController()

    const resultFunction = async (opts: TimeoutOpts): Promise<Messages.RESULT_MSG> =>
      new Promise<Messages.RESULT_MSG>((resolve) => {
        opts.signal.addEventListener('abort', () => {
          abortCalled = true
        })
        setTimeout(() => {
          resolve(Messages.RESULT_MSG)
        }, 50)
      })
    {
    }

    const result = abortableTimeout(resultFunction, Messages.ABORT_MSG, Messages.TIMEOUT_MSG, {
      timeout: 100,
      signal: abort.signal
    })

    assert((await result) === Messages.RESULT_MSG)

    await new Promise<void>((resolve) => setTimeout(resolve, 150))

    assert(!abortCalled)

    // Make sure that the outer abort is no longer forwarded
    abort.abort()

    assert(!abortCalled)
  })

  it('abort after timeout', async function () {
    let abortCalls = 0
    let abort = new AbortController()

    const result = abortableTimeout(
      // Promise that does not resolve
      (opts: TimeoutOpts): Promise<Messages.RESULT_MSG> =>
        new Promise<Messages.RESULT_MSG>(() => {
          opts.signal.addEventListener('abort', () => {
            abortCalls++
          })
        }),
      Messages.ABORT_MSG,
      Messages.TIMEOUT_MSG,
      {
        timeout: 50,
        signal: abort.signal
      }
    )

    assert((await result) === Messages.TIMEOUT_MSG)

    abort.abort()

    assert(abortCalls == 1)
  })

  it('timeout after abort', async function () {
    let abortCalls = 0
    let abort = new AbortController()

    const result = abortableTimeout(
      // Promise that does not resolve
      (opts: TimeoutOpts): Promise<Messages.RESULT_MSG> =>
        new Promise<Messages.RESULT_MSG>(() => {
          opts.signal.addEventListener('abort', () => {
            abortCalls++
          })
        }),
      Messages.ABORT_MSG,
      Messages.TIMEOUT_MSG,
      {
        timeout: 50,
        signal: abort.signal
      }
    )

    abort.abort()

    assert((await result) === Messages.ABORT_MSG)

    abort.abort()

    assert(abortCalls == 1)
  })

  it('throw before timeout', async function () {
    let abortCalls = 0
    let abort = new AbortController()

    const result = abortableTimeout(
      // Produce immediately an error
      (opts: TimeoutOpts): Promise<Messages.RESULT_MSG> =>
        new Promise<Messages.RESULT_MSG>((_, reject) => {
          opts.signal.addEventListener('abort', () => {
            abortCalls++
          })

          reject(Error(`boom`))
        }),
      Messages.ABORT_MSG,
      Messages.TIMEOUT_MSG,
      {
        timeout: 50
      }
    )

    await assert.rejects(result, Error('boom'))

    abort.abort()

    // Give abort time to happen
    await new Promise((resolve) => setTimeout(resolve, 100))

    assert(abortCalls == 0)
  })

  it('throw after timeout', async function () {
    let abortCalls = 0
    let abort = new AbortController()

    const resultFunction = async (opts: TimeoutOpts): Promise<Messages.RESULT_MSG> => {
      opts.signal.addEventListener('abort', () => {
        abortCalls++
      })

      await new Promise((resolve) => setTimeout(resolve, 100))

      throw Error('boom')
    }

    const result = abortableTimeout(resultFunction, Messages.ABORT_MSG, Messages.TIMEOUT_MSG, {
      timeout: 50
    })

    assert((await result) === Messages.TIMEOUT_MSG)

    // Produces an uncaught promise rejection if errors are not handled properly
    await new Promise((resolve) => setTimeout(resolve, 150))

    abort.abort()

    assert(abortCalls == 1)
  })
})
