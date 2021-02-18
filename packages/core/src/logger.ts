import Metrics from 'libp2p/src/metrics'
import PeerId from 'peer-id'

import { durations } from '@hoprnet/hopr-utils'
// import { logDebugData } from '@hoprnet/hopr-logs'

const DEFAULT_DEBUG_INTERVAL = durations.seconds(20)

/**
 * Basic logger to monitor mechanisms within HOPR
 */
export class Logger {
  private interval: NodeJS.Timeout

  private uncaughtExceptionListener: (err: any) => void
  private unhandledRejectionListener: (reason: any, promise: Promise<any>) => void
  private warningListener: (err: any) => void

  constructor(private metrics: Metrics, private debugLogger = console.log) {
    this.uncaughtExceptionListener = this.logUncaughtException.bind(this)
    this.unhandledRejectionListener = this.logUnhandledPromiseRejection.bind(this)
    this.warningListener = this.logWarning.bind(this)
  }

  public start() {
    this.interval = setInterval(this.createConnectivityLog.bind(this), DEFAULT_DEBUG_INTERVAL)

    process.prependListener('warning', this.warningListener)
    process.prependListener('uncaughtException', this.uncaughtExceptionListener)
    process.prependListener('unhandledRejection', this.unhandledRejectionListener)
  }

  public stop() {
    clearInterval(this.interval)

    process.off('warning', this.warningListener)
    process.off('uncaughtException', this.uncaughtExceptionListener)
    process.off('unhandledRejection', this.unhandledRejectionListener)
  }

  private logWarning(warning: any): void {
    this.debugLogger({
      warning: {
        message: warning.message ?? 'no message provided',
        stackTrace: warning.stack ?? 'no stacktrace provided'
      }
    })
  }

  private logUncaughtException(err: any): void {
    if (err.stack?.match(/UnhandledPromiseRejection/)) {
      // already caught be rejection handler
      return
    }

    this.debugLogger({
      error: {
        message: err.message ?? 'no message provided',
        stackTrace: err.stack ?? 'no stacktrace provided'
      }
    })
  }

  private logUnhandledPromiseRejection(reason: any, promise: Promise<any>): void {
    this.debugLogger({
      rejectedPromise: {
        rawPromise: promise?.toString() ?? '',
        reason,
        message: reason.message ?? 'no message provided',
        stackTrace: reason.stack ?? 'no stacktrace provided'
      }
    })
  }

  private createConnectivityLog() {
    const report = { connectionReport: {} }

    for (const peer of this.metrics.peers) {
      try {
        let peerId = PeerId.createFromB58String(peer)
        try {
          peerId = PeerId.createFromB58String(peer)
        } catch (err) {
          // Necessary because libp2p-metrics uses placeholder as
          // long as the correct PeerId is unknown
          continue
        }

        Object.assign(report.connectionReport, {
          [peer]: this.metrics.forPeer(peerId)?.toJSON().movingAverages
        })
      } catch (err) {
        console.log(err)
      }
    }

    this.debugLogger(report)
  }
}
