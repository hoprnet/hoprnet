import Hopr from '@hoprnet/hopr-core'
import http from 'http'
import fs from 'fs'
import ws from 'ws'
import path from 'path'
import debug from 'debug'
import { parse } from 'url'
import next from 'next'
import type { Server } from 'http'
import stripAnsi from 'strip-ansi'
import { LogStream } from './logs'
import { NODE_ENV } from './env'
import { Balance, NativeBalance, SUGGESTED_BALANCE, SUGGESTED_NATIVE_BALANCE } from '@hoprnet/hopr-utils'
import { Commands } from './commands'
import cookie from 'cookie'

let debugLog = debug('hoprd:admin')

const MIN_BALANCE = new Balance(SUGGESTED_BALANCE).toFormattedString()
const MIN_NATIVE_BALANCE = new NativeBalance(SUGGESTED_NATIVE_BALANCE).toFormattedString()

export class AdminServer {
  private app: any
  private server: Server | undefined
  private node: Hopr | undefined
  private wsServer: any
  private cmds: Commands

  constructor(private logs: LogStream, private host: string, private port: number, private apiToken?: string) {}

  authenticate(req): boolean {
    if (!this.apiToken) {
      this.logs.log('ws client connected [ authentication DISABLED ]')
      return true
    }

    if (req.headers.cookie == undefined) {
      return false
    }

    if (req.url) {
      const query = parse(req.url).query
      const [_, apiToken] = (query && query.split('=')) || []
      if (apiToken == this.apiToken) {
        return true
      }
    }

    let cookies: ReturnType<typeof cookie.parse> | undefined
    try {
      cookies = cookie.parse(req.headers.cookie)
    } catch (e) {
      this.logs.error(`failed parsing cookies`, e)
    }

    if (!cookies || cookies['X-Auth-Token'] !== this.apiToken) {
      this.logs.log('ws client failed authentication')
      return false
    }
    this.logs.log('ws client connected [ authentication ENABLED ]')
    return true
  }

  async setup() {
    let adminPath = path.resolve(__dirname, '../hopr-admin/')
    if (!fs.existsSync(adminPath)) {
      // In Docker
      adminPath = path.resolve(__dirname, './hopr-admin')
    }
    debugLog('using', adminPath)

    this.app = next({
      dev: NODE_ENV === 'development',
      dir: adminPath
    })
    const handle = this.app.getRequestHandler()
    await this.app.prepare()

    this.server = http.createServer((req, res) => {
      const parsedUrl = parse(req.url || '', true)
      handle(req, res, parsedUrl)
    })

    this.server.once('error', (err: any) => {
      console.log(`Failed to start Admin interface`)
      console.log(err)
      process.exit(1)
    })

    this.server.listen(this.port, this.host)
    this.logs.log('Admin server listening on port ' + this.port)

    this.wsServer = new ws.Server({ server: this.server })

    this.wsServer.on('connection', (socket: any, req: any) => {
      if (!this.authenticate(req)) {
        socket.send(
          JSON.stringify({
            type: 'auth-failed',
            msg: 'authentication failed',
            ts: new Date().toISOString()
          })
        )
        socket.close()
        return
      }

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
    })
  }

  registerNode(node: Hopr, cmds: any, settings?: any) {
    this.node = node
    this.cmds = cmds
    if (settings) {
      this.cmds.setState(settings)
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
    connectionReport(this.node, this.logs)
    reportMemoryUsage(this.logs)

    process.env.NODE_ENV == 'production' && showDisclaimer(this.logs)

    this.cmds.execute(() => {}, `alias ${node.getId().toB58String()} me`)
  }
}

const DISCLAIMER = `-- This software is still under development --\n\tFor testing, this node requires ${MIN_NATIVE_BALANCE}, and at least ${MIN_BALANCE} \n\tHowever, do NOT add assets to the node that you can't lose!`

export function showDisclaimer(logs: LogStream) {
  logs.warn(DISCLAIMER)
  setInterval(() => {
    logs.warn(DISCLAIMER)
  }, 60 * 1000)
}

export async function reportMemoryUsage(logs: LogStream) {
  const used = process.memoryUsage()
  const usage = process.resourceUsage()
  debugLog(`Process stats: mem ${used.rss / 1024}k (max: ${usage.maxRSS / 1024}k) ` + `cputime: ${usage.userCPUTime}`)
  setTimeout(() => reportMemoryUsage(logs), 60_000)
}

export async function connectionReport(node: Hopr, logs: LogStream) {
  logs.logConnectedPeers(node.getConnectedPeers().map((p) => p.toB58String()))
  setTimeout(() => connectionReport(node, logs), 60_000)
}
