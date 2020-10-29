import ws from 'ws'
import debug from 'debug'

export type Socket = ws

let debugLog = debug('hoprd')

const MAX_MESSAGES_CACHED = 100

type Message = {
  type: string
  msg: string
  ts: string
}

//
// @implements LoggerService of nestjs
export class LogStream {
  private messages: Message[] = []
  private connections: Socket[] = []

  constructor() {}

  subscribe(sock: Socket) {
    this.connections.push(sock)
    this.messages.forEach((m) => this._sendMessage(m, sock))
  }

  log(...args: string[]) {
    const msg = {type: 'log', msg: `${args.join(' ')}`, ts: new Date().toISOString()}
    this._log(msg)
  }

  error(message: string, trace: string) {
    this.log(message)
  }

  logFatalError(message: string) {
    const msg = {type: 'fatal-error', msg: message, ts: new Date().toISOString()}
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

  logFullLine(...args: string[]) {
    const msg = {type: 'log', msg: `${args.join(' ')}`, ts: new Date().toISOString()}
    this._log(msg)
  }

  logConnectedPeers(peers: string[]) {
    const msg = {type: 'connected', msg: peers.join(','), ts: new Date().toISOString()}
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
