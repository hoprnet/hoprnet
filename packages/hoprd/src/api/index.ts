import { WebSocketServer } from 'ws'
import express from 'express'
import http from 'http'
import { debug, Hopr, HoprdPersistentDatabase, MessageInbox } from '@hoprnet/hopr-utils'
import * as api from './v3.js'

import type { StateOps } from '../types.js'

const debugLog = debug('hoprd:api')

export default function setupAPI(
  node: Hopr,
  inbox: MessageInbox,
  stateOps: StateOps,
  options: {
    disableApiAuthentication: boolean
    apiHost: string
    apiPort: number
    apiToken?: string
  },
  db: HoprdPersistentDatabase
): () => void {
  debugLog('Enabling Rest API v3 and WS API v3')
  const service = express()
  const server = http.createServer(service)

  api.setupRestApi(service, '/api/v3', node, inbox, stateOps, options, db)
  api.setupWsApi(server, new WebSocketServer({ noServer: true }), node, options, db)

  return function listen() {
    server
      .listen(options.apiPort, options.apiHost, () => {
        debugLog(`API server on ${options.apiHost} listening on port ${options.apiPort}`)
      })
      .on('error', (err: any) => {
        debugLog(`Failed to start API server: ${err}`)

        // bail out, fail hard because we cannot proceed with the overall
        // startup
        throw err
      })
  }
}
