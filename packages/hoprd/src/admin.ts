import Hopr, { SUGGESTED_NATIVE_BALANCE } from '@hoprnet/hopr-core'
import BN from 'bn.js'
import http from 'http'
import fs from 'fs'
import ws from 'ws'
import path from 'path'
import { parse } from 'url'
import next from 'next'
import type { Server } from 'http'
import stripAnsi from 'strip-ansi'
import { LogStream } from './logs'
import { NODE_ENV } from './env'
import { Logger, Balance, NativeBalance } from '@hoprnet/hopr-utils'

const log: Logger = Logger.getLogger('hoprd.admin')

export class AdminServer {
  private app: any
  private server: Server | undefined
  private node: Hopr | undefined
  private wsServer: any
  private cmds: any

  constructor(private logAdmin: LogStream, private host: string, private port: number) {}

  async setup() {
    let adminPath = path.resolve(__dirname, '../hopr-admin/')
    if (!fs.existsSync(adminPath)) {
      // In Docker
      adminPath = path.resolve(__dirname, './hopr-admin')
    }
    log.info('using', adminPath)

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
      log.error(`Failed to start Admin interface`, err)
      process.exit(1)
    })

    this.server.listen(this.port, this.host)
    this.logAdmin.log('Admin server listening on port ' + this.port)

    this.wsServer = new ws.Server({ server: this.server })

    this.wsServer.on('connection', (socket: any) => {
      socket.on('message', (message: string) => {
        log.info('Message from client', message)
        this.logAdmin.logFullLine(`admin > ${message}`)
        if (this.cmds) {
          this.cmds.execute(message.toString()).then((resp: any) => {
            if (resp) {
              // Strings may have ansi stuff in it, get rid of it:
              resp = stripAnsi(resp)
              this.logAdmin.logFullLine(resp)
            }
          })
        }
        // TODO
      })
      socket.on('error', (err: string) => {
        log.error('Websocket error', err)
        this.logAdmin.log('Websocket error', err.toString())
      })
      this.logAdmin.subscribe(socket)
    })
  }

  registerNode(node: Hopr, cmds: any, settings?: any) {
    this.node = node
    this.cmds = cmds
    if (settings) {
      this.cmds.setState(settings)
    }

    this.node.on('hopr:channel:opened', (channel) => {
      this.logAdmin.log(`Opened channel to ${channel[0].toB58String()}`)
    })

    this.node.on('hopr:channel:closed', (peer) => {
      this.logAdmin.log(`Closed channel to ${peer.toB58String()}`)
    })

    this.node.on('hopr:warning:unfunded', (addr) => {
      const min = new Balance(new BN(0)).toFormattedString.apply(SUGGESTED_NATIVE_BALANCE)

      this.logAdmin.log(
        `- The account associated with this node has no ${Balance.SYMBOL},\n` +
          `  in order to send messages, or open channels, you will need to send` +
          `  at least ${min} to ${addr}`
      )
    })

    this.node.on('hopr:warning:unfundedNative', (addr) => {
      const min = new NativeBalance(new BN(0)).toFormattedString.apply(SUGGESTED_NATIVE_BALANCE)

      this.logAdmin.log(
        `- The account associated with this node has no ${NativeBalance.SYMBOL},\n` +
          `  in order to fund gas for protocol overhead you will need to send\n` +
          `  ${min} to ${addr}`
      )
    })

    // Setup some noise
    connectionReport(this.node, this.logAdmin)
    reportMemoryUsage(this.logAdmin)

    process.env.NODE_ENV == 'production' && showDisclaimer(this.logAdmin)

    this.cmds.execute(`alias ${node.getId().toB58String()} me`)
  }
}

const DISCLAIMER = `-- This software is still under development --\n\tFor testing, this node requires 1 xDAI, and at least 10 wxHOPR \n\tHowever, do NOT add assets to the node that you can't lose`

export function showDisclaimer(logs: LogStream) {
  logs.warn(DISCLAIMER)
  setInterval(() => {
    logs.warn(DISCLAIMER)
  }, 60 * 1000)
}

export async function reportMemoryUsage(logs: LogStream) {
  const used = process.memoryUsage()
  const usage = process.resourceUsage()
  log.info(`Process stats: mem ${used.rss / 1024}k (max: ${usage.maxRSS / 1024}k) ` + `cputime: ${usage.userCPUTime}`)
  setTimeout(() => reportMemoryUsage(logs), 60_000)
}

export async function connectionReport(node: Hopr, logs: LogStream) {
  logs.logConnectedPeers(node.getConnectedPeers().map((p) => p.toB58String()))
  setTimeout(() => connectionReport(node, logs), 60_000)
}
