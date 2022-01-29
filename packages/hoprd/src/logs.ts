import ws from 'ws'
import { debug } from '@hoprnet/hopr-utils'
import RunQueue from 'run-queue'
import { randomBytes } from 'crypto'
import { Ed25519Provider } from 'key-did-provider-ed25519'
import KeyResolver from 'key-did-resolver'
import { DID } from 'dids'
import CeramicClient from '@ceramicnetwork/http-client'
import { TileDocument } from '@ceramicnetwork/stream-tile'

export type Socket = ws

// ensure log including JSON objects is always printed on a single line
const debugBase = debug('hoprd')
const debugLog = (msg) => debugBase('%o', msg)

const MAX_MESSAGES_CACHED = 100

class Queue {
  private queue: RunQueue = undefined
  private completed: boolean = false
  private opts: {
    maxConcurrency: number
  } = undefined

  constructor(opts) {
    this.opts = opts
    this.queue = new RunQueue(opts)
  }
  add(priority, job) {
    if (this.completed) {
      this.queue = new RunQueue(this.opts)
      this.completed = false
    }
    this.queue.add(priority, job)
  }
  async run() {
    this.queue.run().then(() => (this.completed = true))
  }
}

const queue = new Queue({ maxConcurrency: 1 })

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
  private did: DID = undefined
  private isPubliclyLogging: boolean = false
  private logClient: CeramicClient = undefined
  private publicLogs: TileDocument = undefined

  constructor(publicLogs = false) {
    this.isPubliclyLogging = publicLogs
    if (this.isPubliclyLogging) {
      this.did = this._setupDid()
    }
  }

  subscribe(sock: Socket) {
    this.connections.push(sock)
    this.messages.forEach((m) => this._sendMessage(m, sock))
  }

  log(...args: string[]) {
    const msg = { type: 'log', msg: `${args.join(' ')}`, ts: new Date().toISOString() }
    this._log(msg)
  }

  error(message: string, trace: string) {
    this.log(message)
    this.log(trace)
  }

  logFatalError(message: string) {
    const msg = { type: 'fatal-error', msg: message, ts: new Date().toISOString() }
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
    const msg = { type: 'status', msg: status, ts: new Date().toISOString() }
    this._log(msg)
  }

  logFullLine(...args: string[]) {
    const msg = { type: 'log', msg: args.join(' '), ts: new Date().toISOString() }
    this._log(msg)
  }

  logConnectedPeers(peers: string[]) {
    const msg = { type: 'connected', msg: peers.join(','), ts: new Date().toISOString() }
    this._log(msg)
  }

  logMessage(...args: string[]) {
    const msg = { type: 'message', msg: args.join(' '), ts: new Date().toISOString() }
    this._log(msg)
  }

  enablePublicLoggingNode = async (loggingProviderUrl) => {
    if (!this.did) {
      throw Error('Public logging is trying to be enabled but no unique DID was found.')
    }
    await this.did.authenticate()
    this.logClient = new CeramicClient(loggingProviderUrl)
    this.logClient.setDID(this.did)
    this.publicLogs = await TileDocument.create(this.logClient, {})
    debugLog(`Public log entry created, see logs at http://documint.net/${this.publicLogs.id.toString()}`)
    return this.publicLogs.id.toString()
  }

  isReadyForPublicLogging = () => this.isPubliclyLogging && this.did

  startLoggingQueue = () => setInterval(() => queue.run(), 5000)

  appendToPublicLogs = async (msg: Message) => {
    const publicLogs = await TileDocument.load(this.logClient, this.publicLogs.id)
    const newPublicLogsContent = Object.assign({}, publicLogs.content, { [Date.now()]: msg })
    await publicLogs.update(newPublicLogsContent)
  }

  _setupDid(): DID {
    const secretKey = Uint8Array.from(randomBytes(32))
    const provider = new Ed25519Provider(secretKey)
    const did = new DID({ provider, resolver: KeyResolver.getResolver() })
    return did
  }

  _log(msg: Message) {
    if (this.isPubliclyLogging) {
      queue.add(0, async () => await this.appendToPublicLogs(msg))
    }
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
