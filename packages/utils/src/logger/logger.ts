/**
 * This is a log4js wrapper module, in case we want to use another logging library in the future
 * Method signature could be made more generalized if need be
 */
// We NEED to use 'log4js-api' because this is an npm library that others will consume.
// This is to avoid dependency hell: we need to leave _them_ the responsiblity for configuring log4js.
// That also means that only the top package with no HOPR package dependency will use `configure` from 'lo4js'.
// In other words, only `hoprd` package will use 'configure', the other will only use lo4js-api dependency.
import { getLogger, Logger as Log4jsLogger } from '@log4js-node/log4js-api'
import { configure as log4jsConfigure } from 'log4js'
import { Configuration as Log4jsConfiguration } from 'log4js'
import { Appender as Log4jsAppender } from 'log4js'

export class Logger {
  private logger: Log4jsLogger

  private constructor(category?: string) {
    this.logger = getLogger(category)
  }

  static getLogger(category?: string): Logger {
    return new Logger(category)
  }

  trace(message: any): void {
    this.logger.trace(message)
  }
  debug(message: any): void {
    this.logger.debug(message)
  }
  info(message: any): void {
    this.logger.info(message)
  }
  warn(message: any): void {
    this.logger.warn(message)
  }
  error(message: any): void {
    this.logger.error(message)
  }
  fatal(message: any): void {
    this.logger.fatal(message)
  }
}

// for hoprd only
export type Configuration = Log4jsConfiguration
export type Appender = Log4jsAppender
export function configure(config: Configuration): void {
  log4jsConfigure(config)
}
