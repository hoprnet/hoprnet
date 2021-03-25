import Hopr, { SUGGESTED_NATIVE_BALANCE } from '@hoprnet/hopr-core'
import BN from 'bn.js'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
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

let debugLog = debug('hoprd:admin')

export class AdminServer {
  private app: any
  private server: Server | undefined
  private node: Hopr<HoprCoreConnector> | undefined
  private wsServer: any
  private cmds: any

  constructor(private logs: LogStream, private host: string, private port: number) {}

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
  }

  registerNode(node: Hopr<HoprCoreConnector>, cmds: any, settings?: any) {
    this.node = node
    this.cmds = cmds
    if (settings) {
      this.cmds.setState(settings)
    }

    this.wsServer = new ws.Server({ server: this.server })

    this.wsServer.on('connection', (socket: any) => {
      socket.on('message', (message: string) => {
        debugLog('Message from client', message)
        this.logs.logFullLine(`admin > ${message}`)
        if (this.cmds) {
          this.cmds.execute(message.toString()).then((resp: any) => {
            if (resp) {
              // Strings may have ansi stuff in it, get rid of it:
              resp = stripAnsi(resp)
              this.logs.logFullLine(resp)
            }
          })
        }
        // TODO
      })
      socket.on('error', (err: string) => {
        debugLog('Error', err)
        this.logs.log('Websocket error', err.toString())
      })
      this.logs.subscribe(socket)
    })

    this.node.on('hopr:crawl:completed', () => {
      this.logs.log('Crawled network')
    })

    this.node.on('hopr:channel:opened', (channel) => {
      this.logs.log(`Opened channel to ${channel[0].toB58String()}`)
    })

    this.node.on('hopr:channel:closed', (peer) => {
      this.logs.log(`Closed channel to ${peer.toB58String()}`)
    })

    this.node.on('hopr:warning:unfunded', (addr) => {
      const min = new node.paymentChannels.types.Balance(new BN(0)).toFormattedString.apply(SUGGESTED_NATIVE_BALANCE)
      this.logs.log(
        `- The account associated with this node has no funds,\n` +
          `  in order to send messages, or open channels, you will need to send` +
          `  at least ${min} to ${addr}`
      )
    })

    this.node.on('hopr:warning:unfundedNative', (addr) => {
      this.logs.log(
        `- The account associated with this node has no gETH,\n` +
          `  in order to fund gas for protocol overhead you will need to send\n` +
          `  0.025 gETH to ${addr}`
      )
    })

    // Setup some noise
    connectionReport(this.node, this.logs)
    reportMemoryUsage(this.logs)

    this.cmds.execute(`alias ${node.getId().toB58String()} me`)
    if (node.bootstrapServers.length == 1) {
      this.cmds.execute(`alias ${node.bootstrapServers[0].getPeerId()} bootstrap`)
    } else {
      node.bootstrapServers.forEach((x, i) => {
        this.cmds.execute(`alias ${x.getPeerId()}  bootstrap${i}`)
      })
    }
  }
}

export async function reportMemoryUsage(logs: LogStream) {
  const used = process.memoryUsage()
  const usage = process.resourceUsage()
  debugLog(`Process stats: mem ${used.rss / 1024}k (max: ${usage.maxRSS / 1024}k) ` + `cputime: ${usage.userCPUTime}`)
  setTimeout(() => reportMemoryUsage(logs), 60_000)
}

export async function connectionReport(node: Hopr<HoprCoreConnector>, logs: LogStream) {
  logs.logConnectedPeers(node.getConnectedPeers().map((p) => p.toB58String()))
  setTimeout(() => connectionReport(node, logs), 60_000)
}
