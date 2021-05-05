/**
 * This is a log4js wrapper module, in case we want to use another logging library in the future
 * Method signature could be made more generalized if need be
 * We NEED to use 'log4js-api' because this is an npm library that others will consume.
 * This is to avoid dependency hell: we need to leave _them_ the responsiblity for configuring log4js.
 * That also means that only the top package with no HOPR package dependency will use `configure` from 'lo4js'.
 * In other words, only `hoprd` package will use 'configure', the other will only use lo4js-api dependency.
 */
import { getLogger, Logger as Log4jsLogger } from '@log4js-node/log4js-api'
import { configure as log4jsConfigure } from 'log4js'
import { Configuration as Log4jsConfiguration } from 'log4js'
import Debug from 'debug'

/** for hoprd only */
type Configuration = Log4jsConfiguration
function configure(config: Configuration): void {
  log4jsConfigure(config)
}

export const setupConfigLogger = () => {
  configure({
    appenders: { out: { type: 'stdout' } },
    categories: { default: { appenders: ['out'], level: 'debug' } }
    // appenders: { custom: { type: ipfsAppender } },
    // categories: { default: { appenders: ['custom'], level: 'debug' } }
  })
}

export abstract class Logger {
  static getLogger(category?: string, useDebug = true): ConfigLogger | DebugLogger {
    return useDebug ? new DebugLogger(category.replace('.', ':')) : new ConfigLogger(category)
  }
  abstract trace(message: unknown, ...args: unknown[]): void
  abstract fatal(message: unknown, ...args: unknown[]): void
  abstract error(message: unknown, ...args: unknown[]): void
  abstract warn(message: unknown, ...args: unknown[]): void
  abstract info(message: unknown, ...args: unknown[]): void
  abstract debug(message: unknown, ...args: unknown[]): void
}
class DebugLogger extends Logger {
  private readonly logger: Debug.Debugger

  public constructor(category?: string) {
    super()
    this.logger = Debug('hopr')
    this.logger.extend(category)
  }

  protected log(message: unknown, ...args: unknown[]): void {
    if (args?.length) this.logger.log(message, args)
    else this.logger.log(message)
  }

  public debug(message: unknown, ...args: unknown[]): void {
    this.logger.log = console.debug.bind(console)
    this.log(message, args)
  }

  public error(message: unknown, ...args: unknown[]): void {
    this.logger.log = console.error.bind(console)
    this.log(message, args)
  }

  public info(message: unknown, ...args: unknown[]): void {
    this.logger.log = console.log.bind(console)
    this.log(message, args)
  }

  public warn(message: unknown, ...args: unknown[]): void {
    this.logger.log = console.warn.bind(console)
    this.log(message, args)
  }

  public fatal(message: unknown, ...args: unknown[]): void {
    this.logger.log = console.error.bind(console)
    this.log(message, args)
  }

  public trace(message: unknown, ...args: unknown[]): void {
    this.logger.log = console.trace.bind(console)
    this.log(message, args)
  }
}

class ConfigLogger extends Logger {
  private readonly logger: Log4jsLogger

  public constructor(category?: string) {
    super()
    this.logger = getLogger(category)
  }

  /**
   * By default, log4js adds an ugly empty list '[]' to the logs when the arg list is empty...
   **/
  public trace(message: unknown, ...args: unknown[]): void {
    if (args?.length) this.logger.trace(message, args)
    else this.logger.trace(message)
  }
  public debug(message: unknown, ...args: unknown[]): void {
    if (args?.length) this.logger.debug(message, args)
    else this.logger.debug(message)
  }
  public info(message: unknown, ...args: unknown[]): void {
    if (args?.length) this.logger.info(message, args)
    else this.logger.info(message)
  }
  public warn(message: unknown, ...args: unknown[]): void {
    if (args?.length) this.logger.warn(message, args)
    else this.logger.warn(message)
  }
  public error(message: unknown, ...args: unknown[]): void {
    if (args?.length) this.logger.error(message, args)
    else this.logger.error(message)
  }
  public fatal(message: unknown, ...args: unknown[]): void {
    if (args?.length) this.logger.fatal(message, args)
    else this.logger.fatal(message)
  }
}
