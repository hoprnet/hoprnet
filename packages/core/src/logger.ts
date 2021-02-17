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
  constructor(private metrics: Metrics, private debugLogger = console.log) {}

  public start() {
    this.interval = setInterval(this.createConnectivityLog.bind(this), DEFAULT_DEBUG_INTERVAL)

    process.on('warning', this.logWarning.bind(this))
    process.on('uncaughtException', this.logUncaughtException.bind(this))
    process.on('unhandledRejection', this.logUnhandledPromiseRejection.bind(this))
  }

  public stop() {
    clearInterval(this.interval)

    process.off('warning', this.logWarning.bind(this))
    process.off('uncaughtException', this.logUncaughtException.bind(this))
    process.off('unhandledRejection', this.logUnhandledPromiseRejection.bind(this))
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
