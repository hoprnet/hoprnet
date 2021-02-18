import PeerId from 'peer-id'
import { Logger } from './logger'
import Metrics from 'libp2p/src/metrics'
import assert from 'assert'

/**
 * @dev this code won't work in the browser
 */
describe('test logger', function () {
  let pId: PeerId
  const recordedEvents: any[] = []

  function fakeMetrics() {
    return {
      forPeer(peer: PeerId) {
        if (peer.equals(pId)) {
          return undefined
        }

        return {
          toJSON() {
            return {
              movingAverages: { '12345': 12345 }
            }
          }
        }
      }
    } as Metrics
  }

  before(async function () {
    pId = await PeerId.create({ keyType: 'secp256k1' })
  })

  beforeEach(function () {
    recordedEvents.splice(0)
  })

  it('should listen to uncaught exceptions', async function () {
    const originalExceptionHandlers = process.listeners('uncaughtException')

    process.removeAllListeners('uncaughtException')

    const logger = new Logger(fakeMetrics(), recordedEvents.push.bind(recordedEvents))

    logger.start()

    process.nextTick(function () {
      throw Error('Test error. Nothing bad happened')
    })

    await new Promise((resolve) => setTimeout(resolve, 100))

    // Restore previous listeners
    for (const exceptionListener of originalExceptionHandlers) {
      process.addListener('uncaughtException', exceptionListener)
    }

    logger.stop()

    assert(
      recordedEvents.length == 1 &&
        recordedEvents.some((recordedEvent) => recordedEvent.error?.message.match(/Test error. Nothing bad happened/)),
      'Logger should log uncaught exception'
    )
  })

  it('should listen to rejected exceptions', async function () {
    // Backup previous listeners
    const originalRejectionHandlers = process.listeners('unhandledRejection')
    process.removeAllListeners('unhandledRejection')

    const originalExceptionHandlers = process.listeners('uncaughtException')
    process.removeAllListeners('uncaughtException')

    const logger = new Logger(fakeMetrics(), recordedEvents.push.bind(recordedEvents))

    logger.start()

    process.nextTick(function () {
      new Promise((_, reject) => process.nextTick(() => reject('Test rejection. Nothing bad happened')))
    })

    await new Promise((resolve) => setTimeout(resolve, 100))

    logger.stop()

    // Restore previous listeners
    for (const exceptionListener of originalExceptionHandlers) {
      process.addListener('uncaughtException', exceptionListener)
    }
    for (const rejectionHandler of originalRejectionHandlers) {
      process.addListener('unhandledRejection', rejectionHandler)
    }

    assert(
      recordedEvents.length == 1 &&
        recordedEvents.some((recordedEvent) =>
          recordedEvent.rejectedPromise?.reason.match(/Test rejection. Nothing bad happened/)
        ),
      'Logger should log unhandled rejected promise'
    )
  })

  it('should catch warnings', async function () {
    // Backup previous listeners
    const originalWarningHandlers = process.listeners('warning')
    process.removeAllListeners('warning')

    const logger = new Logger(fakeMetrics(), recordedEvents.push.bind(recordedEvents))

    logger.start()

    process.nextTick(function () {
      process.emitWarning('Test warning. Nothing bad happened')
    })

    await new Promise((resolve) => setTimeout(resolve, 100))

    logger.stop()

    // Restore previous listeners
    for (const warningListener of originalWarningHandlers) {
      process.addListener('uncaughtException', warningListener)
    }

    assert(
      recordedEvents.length == 1 &&
        recordedEvents.some((recordedEvent) =>
          recordedEvent.warning?.message.match(/Test warning. Nothing bad happened/)
        ),
      'Logger should log unhandled rejected promise'
    )
  })
})
