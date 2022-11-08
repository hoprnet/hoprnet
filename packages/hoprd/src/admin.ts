import type Hopr from '@hoprnet/hopr-core'
import type { Duplex } from 'stream'
import http from 'http'
import fs from 'fs'
import path from 'path'
import { parse } from 'url'
import { default as next } from 'next'
import type { Server as HttpServer } from 'http'
import type { LogStream } from './logs.js'
import { NODE_ENV } from './env.js'
import {
  Balance,
  NativeBalance,
  SUGGESTED_BALANCE,
  SUGGESTED_NATIVE_BALANCE,
  debug,
  startResourceUsageLogger,
  retimer
} from '@hoprnet/hopr-utils'
import type { WebSocket } from 'ws'

let debugLog = debug('hoprd:admin')

const MIN_BALANCE = new Balance(SUGGESTED_BALANCE).toFormattedString()
const MIN_NATIVE_BALANCE = new NativeBalance(SUGGESTED_NATIVE_BALANCE).toFormattedString()

/**
 * Server that hosts hopr-admin website
 */
export class AdminServer {
  private app: ReturnType<typeof next>
  public server: HttpServer | undefined
  private node: Hopr | undefined

  constructor(private logs: LogStream, private host: string, private port: number) {}

  setup(): Promise<void> {
    return new Promise(async (resolve, reject) => {
      let adminPath: string
      for (const adminRelPath of ['../hopr-admin', './hopr-admin']) {
        const adminPathInt = new URL(adminRelPath, import.meta.url).pathname
        const nextPath = path.resolve(adminPathInt, '.next')
        if (!fs.existsSync(nextPath)) {
          continue
        }
        adminPath = adminPathInt
        break
      }

      if (!adminPath) {
        console.log('Failed to start Admin interface')
        return reject(Error(`could not find NextJS app`))
      }

      debugLog('using', adminPath)

      const nextConfig = {
        dev: NODE_ENV === 'development',
        dir: adminPath
      } as any

      if (NODE_ENV === 'development') {
        nextConfig.conf = {
          distDir: `build/${this.port}`
        }
      }

      this.app = next(nextConfig)
      const handle = this.app.getRequestHandler()
      await this.app.prepare()

      this.server = http.createServer((req, res) => {
        const parsedUrl = parse(req.url || '', true)
        handle(req, res, parsedUrl)
      })

      this.server.once('error', (err: any) => {
        console.log('Failed to start Admin interface')
        reject(err)
      })

      // Handles error resulting from broken client connections.
      // see https://nodejs.org/dist/latest-v16.x/docs/api/http.html#event-clienterror
      this.server.on('clientError', (err: Error, socket: Duplex) => {
        if ((err as any).code === 'ECONNRESET' || !socket.writable) {
          return
        }

        // End the socket
        socket.end('HTTP/1.1 400 Bad Request\r\n\r\n')
      })

      this.server.listen(this.port, this.host, resolve)
      this.logs.log('Admin server listening on port ' + this.port)
    })
  }

  registerNode(node: Hopr) {
    this.node = node

    this.node.on('hopr:channel:opened', (channel) => {
      this.logs.log(`Opened channel to ${channel[0].toString()}`)
    })

    this.node.on('hopr:channel:closed', (peer) => {
      this.logs.log(`Closed channel to ${peer.toString()}`)
    })

    this.node.on('hopr:warning:unfunded', (addr) => {
      this.logs.log(
        `- The account associated with this node has no ${Balance.SYMBOL},\n` +
          `  in order to send messages, or open channels, you will need to send` +
          `  at least ${MIN_BALANCE} to ${addr}`
      )
    })

    this.node.on('hopr:warning:unfundedNative', (addr) => {
      this.logs.log(
        `- The account associated with this node has no ${NativeBalance.SYMBOL},\n` +
          `  in order to fund gas for protocol overhead you will need to send\n` +
          `  ${MIN_NATIVE_BALANCE} to ${addr}`
      )
    })

    this.logs.logStatus(this.node.status === 'RUNNING' ? 'READY' : 'PENDING')

    // Setup some noise
    startConnectionReports(this.node, this.logs)
    startResourceUsageLogger(debugLog)
  }

  public onConnection(socket: WebSocket) {
    socket.on('message', (message: string) => {
      debugLog('Message from client', message)
      this.logs.logFullLine(`admin > ${message}`)
    })

    socket.on('error', (err: string) => {
      debugLog('Error', err)
      this.logs.log('Websocket error', err.toString())
    })
    this.logs.subscribe(socket)
  }
}

export async function startConnectionReports(node: Hopr, logs: LogStream) {
  const printConnectedPeers = () => {
    const peers = node.getConnectedPeers()
    logs.logConnectedPeers(
      (function* () {
        for (const peerId of peers) {
          yield peerId.toString()
        }
      })()
    )
  }

  retimer(printConnectedPeers, () => 60 * 1000)
}
