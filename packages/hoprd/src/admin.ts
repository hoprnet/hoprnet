import Hopr from '@hoprnet/hopr-core'
import type { HoprOptions } from '@hoprnet/hopr-core'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { commands } from '@hoprnet/hopr-chat'
import { LogStream, Socket } from './logs'
import express from 'express'
import http from 'http'
import fs from 'fs'
import ws from 'ws'
import path from 'path'
import debug from 'debug'
import { parse } from 'url'
import next from 'next'
import type { Server } from 'http'
import stripAnsi from 'strip-ansi'

let debugLog = debug('hoprd:admin')

export class AdminServer {
  private app: any
  private server: Server | undefined
  private node: Hopr<HoprCoreConnector> | undefined
  private port: number
  private wsServer: any
  private cmds: any

  constructor(private logs: LogStream) {
    this.port = process.env.HOPR_ADMIN_PORT ? parseInt(process.env.HOPR_ADMIN_PORT) : 3000
  }

  async setup() {
    let adminPath = path.resolve(__dirname, '../hopr-admin/')
    if (!fs.existsSync(adminPath)) {
      // In Docker
      adminPath = path.resolve(__dirname, './hopr-admin')
    }
    debugLog('using', adminPath)

    this.app = next({
      dev: true,
      dir: adminPath,
      conf: {
        devIndicators: {
          autoPrerender: false
        }
      }
    })
    const handle = this.app.getRequestHandler()
    await this.app.prepare()

    this.server = http.createServer((req, res) => {
      const parsedUrl = parse(req.url || '', true)
      handle(req, res, parsedUrl)
    })

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

    this.server.listen(this.port)
    this.logs.log('Admin server listening on port ' + this.port)
  }

  registerNode(node: Hopr<HoprCoreConnector>, settings?: any) {
    this.node = node
    this.cmds = new commands.Commands(node)
    if (settings) {
      this.cmds.setState(settings)
    }

    // Setup some noise
    connectionReport(this.node, this.logs)
    periodicCrawl(this.node, this.logs)
    reportMemoryUsage(this.logs)
  }
}

const CRAWL_TIMEOUT = 100_000 // ~15 mins
export async function periodicCrawl(node: Hopr<HoprCoreConnector>, logs: LogStream) {
  try {
    await node.crawl()
    logs.log('Crawled network')
  } catch (err) {
    logs.log('Failed to crawl')
    logs.log(err)
  }
  setTimeout(() => periodicCrawl(node, logs), CRAWL_TIMEOUT)
}

export async function reportMemoryUsage(logs: LogStream) {
  const used = process.memoryUsage()
  const usage = process.resourceUsage()
  logs.log(`Process stats: mem ${used.rss / 1024}k (max: ${usage.maxRSS / 1024}k) ` + `cputime: ${usage.userCPUTime}`)
  setTimeout(() => reportMemoryUsage(logs), 60_000)
}

export async function connectionReport(node: Hopr<HoprCoreConnector>, logs: LogStream) {
  logs.logConnectedPeers(node.getConnectedPeers().map((p) => p.toB58String()))
  setTimeout(() => connectionReport(node, logs), 60_000)
}
