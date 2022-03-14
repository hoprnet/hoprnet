import express from 'express'
import http from 'http'
import ws from 'ws'
import { debug } from '@hoprnet/hopr-utils'
import * as apiV1 from './v1'
import * as apiV2 from './v2'

import type Hopr from '@hoprnet/hopr-core'
import type { AdminServer } from '../admin'
import type { LogStream } from '../logs'
import type { StateOps } from '../types'

const debugLog = debug('hoprd:api')

export default function setupAPI(
  node: Hopr,
  logs: LogStream,
  stateOps: StateOps,
  options: {
    api: boolean
    apiPort: number
    apiHost: string
    admin: boolean
    adminPort: number
    adminHost: string
    apiToken?: string
  },
  adminServer?: AdminServer // required by WS v1 (hopr-admin)
) {
  // creates server for Rest API v1, v2 and WS v2
  if (options.api) {
    debugLog('Enabling Rest API v1, v2 and WS v2')
    const service = express()
    const server = http.createServer(service)

    // apiV1.setupRestApi(service, '/api/v1', node, logs, stateOps, options)
    apiV2.setupRestApi(service, '/api/v2', node, stateOps, options)
    apiV2.setupWsApi(server, new ws.Server({ noServer: true }), node, logs, options)

    server
      .listen(options.apiPort, options.apiHost, () => {
        logs.log(`API server on ${options.apiHost} listening on port ${options.apiPort}`)
      })
      .on('error', (err: any) => {
        logs.log(`Failed to start API server: ${err}`)

        // bail out, fail hard because we cannot proceed with the overall
        // startup
        throw err
      })
  }

  // deprecated: creates WS v1 server for hopr-admin
  // if (options.admin && adminServer?.server) {
  //   debugLog('Enabling WS v1')
  //   apiV1.setupWsApi(new ws.Server({ server: adminServer.server }), logs, options, adminServer)
  //
  //   logs.log(`deprecated WS admin API server on ${options.adminHost} listening on port ${options.adminPort}`)
  // }
}
