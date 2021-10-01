import debug from 'debug'
import util from 'util';
import pino from 'pino'
import { createWriteStream } from 'pino-logflare'

// create pino-logflare stream
const stream = createWriteStream({
  apiKey: "VgZwc3dsuP6e",
  sourceToken: "e22bd592-6249-4cd2-a653-e3b18853cc53"
});

const logger = pino({}, stream);

const wrappedDebug = (namespace: any) => {
  return (message: any, ...parameters: any[]) => {
    if (process.env.TELEMETRY && process.env.TELEMETRY_ID) {
      const log = util.format(message, ...parameters)
      logger.info({ id: process.env.TELEMETRY_ID, log: log, ts: new Date(Date.now()).toISOString() });
    }
    return debug(namespace)(message, ...parameters)
  }
}

export { wrappedDebug as debug };