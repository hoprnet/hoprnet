import { create_counter } from '../../../hoprd/lib/hoprd_hoprd.js'

// Metrics
const metric_countSuppresedRejections = create_counter(
  'utils_counter_suppressed_unhandled_promise_rejections',
  'Counter of suppressed unhandled promise rejections'
)

/**
 * Sets a custom promise rejection handler to filter out known promise rejections
 * that are harmless but couldn't be handled for some reason.
 */
export function setupPromiseRejectionFilter() {
  // See https://github.com/hoprnet/hoprnet/issues/3755
  process.on('unhandledRejection', (reason: any, _promise: Promise<any>) => {
    if (reason && reason.message && reason.message.toString) {
      const msgString = reason.toString()

      // Only silence very specific errors
      if (
        // HOPR uses the `stream-to-it` library to convert streams from Node.js sockets
        // to async iterables. This library has shown to have issues with runtime errors,
        // mainly ECONNRESET and EPIPE
        msgString.match(/read ETIMEDOUT/) ||
        msgString.match(/read ECONNRESET/) ||
        msgString.match(/write ETIMEDOUT/) ||
        msgString.match(/write ECONNRESET/) ||
        msgString.match(/write EPIPE/) ||
        // Requires changes in libp2p, tbd in upstream PRs to libp2p
        msgString.match(/The operation was aborted/) ||
        // issues with WebRTC socket and `stream-to-it`
        msgString.match(/ERR_DATA_CHANNEL/) ||
        msgString.match(/ERR_ICE_CONNECTION_FAILURE/)
      ) {
        console.error('Unhandled promise rejection silenced')
        metric_countSuppresedRejections.increment()
        return
      }
    }

    console.warn('UnhandledPromiseRejectionWarning')
    console.log(reason)
    process.exit(1)
  })
}
