import type Hopr from '@hoprnet/hopr-core'
import http from 'http'
import fs from 'fs'
import path from 'path'
import { parse } from 'url'
import next from 'next'
import type { Server as HttpServer } from 'http'
import stripAnsi from 'strip-ansi'
import type { LogStream } from './logs'
import { NODE_ENV } from './env'
import {
  Balance,
  NativeBalance,
  SUGGESTED_BALANCE,
  SUGGESTED_NATIVE_BALANCE,
  debug,
  startResourceUsageLogger
} from '@hoprnet/hopr-utils'
import { Commands } from './commands'
import type { WebSocket } from 'ws'

let debugLog = debug('hoprd:admin')

const MIN_BALANCE = new Balance(SUGGESTED_BALANCE).toFormattedString()
const MIN_NATIVE_BALANCE = new NativeBalance(SUGGESTED_NATIVE_BALANCE).toFormattedString()

export class AdminServer {
  private app: ReturnType<typeof next>
  public server: HttpServer | undefined
  private node: Hopr | undefined
  private cmds: Commands

  constructor(private logs: LogStream, private host: string, private port: number) {}

  async setup() {
    let adminPath
    for (const adminRelPath of ['../hopr-admin', './hopr-admin']) {
      const adminPathInt = path.resolve(__dirname, adminRelPath)
      const nextPath = path.resolve(adminPathInt, '.next')
      if (!fs.existsSync(nextPath)) {
        continue
      }
      adminPath = adminPathInt
      break
    }

    if (!adminPath) {
      console.log('Failed to start Admin interface: could not find NextJS app')
      process.exit(1)
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
      console.log(err)
      process.exit(1)
    })

    this.server.listen(this.port, this.host)
    this.logs.log('Admin server listening on port ' + this.port)
  }

  registerNode(node: Hopr, cmds: any, settings?: any) {
    this.node = node
    this.cmds = cmds
    if (settings) {
      this.cmds.stateOps.setState(settings)
    }

    this.node.on('hopr:channel:opened', (channel) => {
      this.logs.log(`Opened channel to ${channel[0].toB58String()}`)
    })

    this.node.on('hopr:channel:closed', (peer) => {
      this.logs.log(`Closed channel to ${peer.toB58String()}`)
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

    process.env.NODE_ENV == 'production' && showDisclaimer(this.logs)

    this.cmds.execute(() => {}, `alias ${node.getId().toB58String()} me`)
  }

  public onConnection(socket: WebSocket) {
    socket.on('message', (message: string) => {
      debugLog('Message from client', message)
      this.logs.logFullLine(`admin > ${message}`)

      if (this.cmds) {
        this.cmds.execute((resp: string) => {
          if (resp) {
            // Strings may have ansi stuff in it, get rid of it:
            resp = stripAnsi(resp)
            this.logs.logFullLine(resp)
          }
        }, message.toString())
      }
    })

    socket.on('error', (err: string) => {
      debugLog('Error', err)
      this.logs.log('Websocket error', err.toString())
    })
    this.logs.subscribe(socket)
  }
}

const DISCLAIMER = `-- This software is still under development --\n\tFor testing, this node requires ${MIN_NATIVE_BALANCE}, and at least ${MIN_BALANCE} \n\tHowever, do NOT add assets to the node that you can't lose!`

export function showDisclaimer(logs: LogStream) {
  logs.warn(DISCLAIMER)
  setInterval(() => {
    logs.warn(DISCLAIMER)
  }, 60 * 1000)
}

export async function startConnectionReports(node: Hopr, logs: LogStream) {
  logs.logConnectedPeers(node.getConnectedPeers().map((p) => p.toB58String()))
  setInterval(() => {
    logs.logConnectedPeers(node.getConnectedPeers().map((p) => p.toB58String()))
  }, 60 * 1000)
}
