import type { default as Hopr } from '@hoprnet/hopr-core'
import type { LogStream } from '../logs.js'
import type { StateOps } from '../types.js'
import express from 'express'
import http from 'http'
import { debug } from '@hoprnet/hopr-utils'
import * as apiV2 from './v2.js'
import { WebSocketServer } from 'ws'
import { MessageInbox } from '../../lib/hoprd_inbox.js'

const debugLog = debug('hoprd:api')

export default function setupAPI(
  node: Hopr,
  inbox: MessageInbox,
  logs: LogStream,
  stateOps: StateOps,
  options: {
    disableApiAuthentication: boolean
    apiHost: string
    apiPort: number
    apiToken?: string
  }
): () => void {
  debugLog('Enabling Rest API v2 and WS API v2')
  const service = express()
  const server = http.createServer(service)

  apiV2.setupRestApi(service, '/api/v2', node, inbox, stateOps, options)
  apiV2.setupWsApi(server, new WebSocketServer({ noServer: true }), node, logs, options)

  return function listen() {
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
}
