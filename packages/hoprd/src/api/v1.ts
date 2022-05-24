import express from 'express'
import bodyParser from 'body-parser'

import type { Application } from 'express'
import type { WebSocketServer } from 'ws'
import type { default as Hopr } from '@hoprnet/hopr-core'
import type { AdminServer } from '../admin.js'

import type { LogStream } from './../logs.js'
import type { StateOps } from '../types.js'
import { Commands } from './../commands/index.js'
import { authenticateWsConnection } from './utils.js'

export function setupRestApi(
  service: Application,
  urlPath: string,
  node: Hopr,
  logs: LogStream,
  stateOps: StateOps,
  options: any
) {
  const router = express.Router()

  router.use(bodyParser.text({ type: '*/*' }))

  router.get('/version', (_, res) => res.send(node.getVersion()))
  router.get('/address/eth', (_, res) => res.send(node.getEthereumAddress().toHex()))
  router.get('/address/hopr', (_, res) => res.send(node.getId().toB58String()))

  const cmds = new Commands(node, stateOps)
  router.post('/command', async (req, res) => {
    await node.waitForRunning()
    logs.log('Node is running')
    if (!options.testNoAuthentication && options.apiToken !== undefined) {
      if (req.headers['x-auth-token'] !== options.apiToken) {
        logs.log('command rejected: authentication failed')
        res.status(403).send('authentication failed')
        return
      }
      logs.log('command accepted: authentication succeeded')
    } else {
      logs.log('command accepted: authentication DISABLED')
    }

    let response = ''
    let log = (s) => {
      response += s
      logs.log(s)
    }
    logs.log('executing API command', req.body, node.status)
    await cmds.execute(log, req.body)
    logs.log('command complete')
    res.send(response)
  })

  service.use(urlPath, router)
}

export function setupWsApi(
  server: WebSocketServer,
  logs: LogStream,
  options: { apiToken?: string },
  adminServer?: AdminServer
) {
  server.on('connection', (socket, req) => {
    const needsAuth = !!options.apiToken
    if (needsAuth && !authenticateWsConnection(req, options.apiToken)) {
      logs.log('ws client failed authentication')
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
    if (!needsAuth) logs.log('ws client connected [ authentication DISABLED ]')
    else logs.log('ws client connected [ authentication ENABLED ]')

    if (adminServer) adminServer.onConnection(socket)
  })
}
