import express from 'express'
import http from 'http'
import ws from 'ws'
import URL from 'url'

import type Hopr from '@hoprnet/hopr-core'
import type { AdminServer } from '../admin'
import type { LogStream } from '../logs'
import type { StateOps } from '../types'
import * as apiV1 from './v1'
import * as apiV2 from './v2'

export default function setupAPI(
  node: Hopr,
  logs: LogStream,
  stateOps: StateOps,
  options: {
    rest: boolean
    restPort: number
    restHost: string
    ws: boolean
    wsPort: number
    wsHost: string
    admin: boolean
    adminPort: number
    adminHost: string
    apiToken?: string
  },
  adminServer?: AdminServer // api V1: required by hopr-admin
) {
  // creates server for rest API v1 and rest API v2
  if (options.rest) {
    const service = express()

    apiV1.setupRestApi(service, '/api/v1', node, logs, stateOps, options)
    apiV2.setupRestApi(service, '/api/v2', node, logs, stateOps, options)

    http
      .createServer(service)
      .listen(options.restPort, options.restHost, () => {
        logs.log(`Rest API server on ${options.restHost} listening on port ${options.restPort}`)
      })
      .on('error', (err: any) => {
        logs.log(`Failed to start Rest API server: ${err}`)

        // bail out, fail hard because we cannot proceed with the overall
        // startup
        throw err
      })
  }

  // creates WS server for API v1 and API v2
  if (options.ws) {
    const server = http.createServer()
    const wsV1 = new ws.Server({ noServer: true, path: '/' })
    const wsV2 = new ws.Server({ noServer: true, path: '/api/v2/messages/websocket' })

    apiV1.setupWsApi(wsV1, logs, options, adminServer)
    apiV2.setupWsApi(wsV2, logs, options)

    server.on('upgrade', (request, socket, head) => {
      const { pathname } = URL.parse(request.url)

      if (pathname === '/') {
        wsV1.handleUpgrade(request, socket, head, function done(ws) {
          wsV1.emit('connection', ws, request)
        })
      } else if (pathname === '/api/v2/messages/websocket') {
        wsV2.handleUpgrade(request, socket, head, function done(ws) {
          wsV2.emit('connection', ws, request)
        })
      } else {
        socket.destroy()
      }
    })

    server
      .listen(options.wsPort, options.wsHost, () => {
        logs.log(`WS API server on ${options.wsHost} listening on port ${options.wsPort}`)
      })
      .on('error', (err: any) => {
        logs.log(`Failed to start WS API server: ${err}`)

        // bail out, fail hard because we cannot proceed with the overall
        // startup
        throw err
      })
  }

  // deprecated: creates WS server for hopr-admin
  if (options.admin && adminServer?.server) {
    const wsV1 = new ws.Server({ server: adminServer.server })
    apiV1.setupWsApi(wsV1, logs, options, adminServer)

    logs.log(`deprecated WS admin API server on ${options.adminHost} listening on port ${options.adminPort}`)
  }
}
