import ws from 'ws'
import { debug } from '@hoprnet/hopr-utils'

export type Socket = ws

// ensure log including JSON objects is always printed on a single line
const debugBase = debug('hoprd')
const debugLog = (msg) => debugBase('%o', msg)

const MAX_MESSAGES_CACHED = 100

// const queue = FIFO<any>()

type Message = {
  type: 'log' | 'fatal-error' | 'status' | 'connected' | 'message'
  msg: string
  ts: string
}

//
// @implements LoggerService of nestjs
export class LogStream {
  private messages: Message[] = []
  private connections: Socket[] = []

  subscribe(sock: Socket) {
    debugLog('WS subscribing socket')
    this.connections.push(sock)
    this.messages.forEach((m) => {
      this._sendMessage(m, sock)
    })
  }

  log(...args: string[]) {
    const msg: Message = { type: 'log', msg: `${args.join(' ')}`, ts: new Date().toISOString() }
    this._log(msg)
  }

  error(message: string, trace: string) {
    this.log(message)
    this.log(trace)
  }

  logFatalError(message: string) {
    const msg: Message = { type: 'fatal-error', msg: message, ts: new Date().toISOString() }
    this._log(msg)
  }

  warn(message: string) {
    this.log('WARN', message)
  }

  debug(message: string) {
    this.log('DEBUG', message)
  }

  verbose(message: string) {
    this.log('VERBOSE', message)
  }

  logStatus(status: 'READY' | 'PENDING') {
    const msg: Message = { type: 'status', msg: status, ts: new Date().toISOString() }
    this._log(msg)
  }

  logFullLine(...args: string[]) {
    const msg: Message = { type: 'log', msg: args.join(' '), ts: new Date().toISOString() }
    this._log(msg)
  }

  logConnectedPeers(peers: Iterable<string>) {
    const it = peers[Symbol.iterator]()
    let chunk = it.next()
    let msg = ''

    while (!chunk.done) {
      msg += chunk.value
      chunk = it.next()
      if (!chunk.done) {
        msg += ', '
      }
    }

    this._log({ type: 'connected', msg, ts: new Date().toISOString() })
  }

  logMessage(...args: string[]) {
    const msg: Message = { type: 'message', msg: args.join(' '), ts: new Date().toISOString() }
    this._log(msg)
  }

  _log(msg: Message) {
    debugLog(msg)
    this.messages.push(msg)
    if (this.messages.length > MAX_MESSAGES_CACHED) {
      // Avoid memory leak
      this.messages.splice(0, this.messages.length - MAX_MESSAGES_CACHED) // delete elements from start
    }
    this.connections.forEach((conn: Socket, i: number) => {
      if (conn.readyState == ws.OPEN) {
        this._sendMessage(msg, conn)
      } else {
        // Handle bad connections:
        if (conn.readyState !== ws.CONNECTING) {
          // Only other possible states are closing or closed
          this.connections.splice(i, 1)
        }
      }
    })
  }

  _sendMessage(m: Message, s: Socket) {
    s.send(JSON.stringify(m))
  }
}
