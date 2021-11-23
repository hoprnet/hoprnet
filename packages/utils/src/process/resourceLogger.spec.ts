import { startResourceUsageLogger } from './resourceLogger'

describe('check resource logger', function () {
  it('should start and stop a resource logger', async function () {
    let firstCall = true

    const TIMEOUT = 50

    const stop = startResourceUsageLogger(() => {
      if (!firstCall) {
        throw Error('fail')
      }
      firstCall = false
    }, TIMEOUT)

    stop()

    await new Promise((resolve) => setTimeout(resolve, 2 * TIMEOUT))

    // produces an uncaught exception when if unregistering the resource logger
    // fails.
  })
})
