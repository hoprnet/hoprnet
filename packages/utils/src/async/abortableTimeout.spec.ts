import type { TimeoutOpts } from './abortableTimeout'

import assert from 'assert'
import AbortController from 'abort-controller'
import { abortableTimeout } from './abortableTimeout'
import { defer } from '../async'

describe('abortable timeout', function () {
  enum Messages {
    ABORT_MSG,
    TIMEOUT_MSG,
    RESULT_MSG
  }

  it('normal timeout behavior', async function () {
    const resultPromise = defer<Messages.RESULT_MSG>()

    const result = abortableTimeout(
      (_opts: TimeoutOpts) => resultPromise.promise,
      Messages.ABORT_MSG,
      Messages.TIMEOUT_MSG,
      {
        timeout: 100
      }
    )

    assert((await result) === Messages.TIMEOUT_MSG)
  })

  it('normal abort behavior', async function () {
    const resultPromise = defer<Messages.RESULT_MSG>()
    const abort = new AbortController()

    const result = abortableTimeout(
      (_opts: TimeoutOpts) => resultPromise.promise,
      Messages.ABORT_MSG,
      Messages.TIMEOUT_MSG,
      {
        timeout: 100,
        abort: abort
      }
    )

    setTimeout(() => abort.abort(), 50)
    assert((await result) === Messages.ABORT_MSG)
  })

  it('normal result behavior', async function () {
    const resultFunction = async (_opts: TimeoutOpts): Promise<Messages.RESULT_MSG> =>
      Promise.resolve(Messages.RESULT_MSG)

    const result = abortableTimeout(resultFunction, Messages.ABORT_MSG, Messages.TIMEOUT_MSG, {
      timeout: 100
    })

    assert((await result) === Messages.RESULT_MSG)
  })

  it('timeout result behavior', async function () {
    const abortCalled = defer<void>()

    const resultFunction = async (opts: TimeoutOpts): Promise<Messages.RESULT_MSG> => {
      return new Promise<Messages.RESULT_MSG>((resolve) => {
        opts.abort.signal.addEventListener('abort', () => abortCalled.resolve())

        setTimeout(() => {
          resolve(Messages.RESULT_MSG)
        }, 150)
      })
    }

    const result = abortableTimeout(resultFunction, Messages.ABORT_MSG, Messages.TIMEOUT_MSG, {
      timeout: 100
    })

    await abortCalled.promise
    assert((await result) === Messages.TIMEOUT_MSG)
  })

  it('aborted result behavior', async function () {
    const abortCalled = defer<void>()
    const abort = new AbortController()

    const resultFunction = async (opts: TimeoutOpts): Promise<Messages.RESULT_MSG> =>
      new Promise<Messages.RESULT_MSG>((resolve) => {
        opts.abort.signal.addEventListener('abort', () => abortCalled.resolve())

        setTimeout(() => {
          resolve(Messages.RESULT_MSG)
        }, 100)
      })
    {
    }

    const result = abortableTimeout(resultFunction, Messages.ABORT_MSG, Messages.TIMEOUT_MSG, {
      timeout: 100,
      abort
    })

    setTimeout(() => abort.abort(), 50)

    await abortCalled.promise
    assert((await result) === Messages.ABORT_MSG)
  })

  it('timeout after result', async function () {
    let abortCalled = false
    const abort = new AbortController()

    abort.signal.addEventListener('abort', () => {
      abortCalled = true
    })

    const resultFunction = async (_opts: TimeoutOpts): Promise<Messages.RESULT_MSG> =>
      new Promise<Messages.RESULT_MSG>((resolve) => {
        setTimeout(() => {
          resolve(Messages.RESULT_MSG)
        }, 50)
      })
    {
    }

    const result = abortableTimeout(resultFunction, Messages.ABORT_MSG, Messages.TIMEOUT_MSG, {
      timeout: 100,
      abort
    })

    assert((await result) === Messages.RESULT_MSG)

    await new Promise<void>((resolve) => setTimeout(resolve, 150))

    assert(!abortCalled)
  })

  it('abort after result', async function () {
    let abort: AbortController

    const resultFunction = (opts: TimeoutOpts): Promise<Messages.RESULT_MSG> =>
      new Promise<Messages.RESULT_MSG>((resolve) => {
        abort = opts.abort
        setTimeout(() => {
          resolve(Messages.RESULT_MSG)
        }, 50)
      })
    {
    }

    const result = abortableTimeout(resultFunction, Messages.ABORT_MSG, Messages.TIMEOUT_MSG, {
      timeout: 100,
      abort
    })

    assert((await result) === Messages.RESULT_MSG)

    assert.doesNotThrow(() => abort.abort())
  })

  it('throw before timeout', async function () {
    const resultFunction = async (_opts: TimeoutOpts): Promise<Messages.RESULT_MSG> => {
      throw Error('boom')
    }

    const result = abortableTimeout(resultFunction, Messages.ABORT_MSG, Messages.TIMEOUT_MSG, {
      timeout: 50
    })

    await assert.rejects(result, {
      message: 'boom'
    })
  })

  it('throw after timeout', async function () {
    const resultFunction = async (_opts: TimeoutOpts): Promise<Messages.RESULT_MSG> => {
      await new Promise((resolve) => setTimeout(resolve, 100))

      throw Error('boom')
    }

    const result = abortableTimeout(resultFunction, Messages.ABORT_MSG, Messages.TIMEOUT_MSG, {
      timeout: 50
    })

    assert((await result) === Messages.TIMEOUT_MSG)

    // Produces an uncaught promise rejection if errors are not handled properly
    await new Promise((resolve) => setTimeout(resolve, 150))
  })
})
