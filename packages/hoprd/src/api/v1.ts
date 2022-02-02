import express from 'express'
import bodyParser from 'body-parser'
import cookie from 'cookie'

import type { Application } from 'express'
import type { WebSocketServer } from 'ws'
import type Hopr from '@hoprnet/hopr-core'
import type { AdminServer } from '../admin'

import type { LogStream } from './../logs'
import type { StateOps } from '../types'
import { Commands } from './../commands'

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
    if (!authenticateConnection(logs, req, options.apiToken)) {
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
    if (adminServer) adminServer.onConnection(socket)
  })
}

const authenticateConnection = (
  logs: LogStream,
  req: { url?: string; headers: Record<any, any> },
  apiToken?: string
): boolean => {
  if (!apiToken) {
    logs.log('ws client connected [ authentication DISABLED ]')
    return true
  }

  // Other clients different to `hopr-admin` might pass the `apiToken` via a
  // query param since they won't be on the same domain the node is hosted,
  // and thus, unable to set the `apiToken` via cookies. Using `req.url` we
  // can detect these cases and provide the ability for any client that
  // knows the `apiToken` to reach your HOPR node.
  if (req.url) {
    try {
      // NB: We use a placeholder domain since req.url only passes query params
      const url = new URL(`https://hoprnet.org${req.url}`)
      const apiToken = url.searchParams?.get('apiToken') || ''
      if (decodeURI(apiToken) == apiToken) {
        logs.log('ws client connected [ authentication ENABLED ]')
        return true
      }
    } catch (e) {
      logs.error('invalid URL queried', e)
    }
  }

  if (req.headers.cookie == undefined) {
    return false
  }

  let cookies: ReturnType<typeof cookie.parse> | undefined
  try {
    cookies = cookie.parse(req.headers.cookie)
  } catch (e) {
    logs.error(`failed parsing cookies`, e)
  }

  if (
    !cookies ||
    (decodeURI(cookies['X-Auth-Token'] || '') !== apiToken && decodeURI(cookies['x-auth-token'] || '') !== apiToken)
  ) {
    logs.log('ws client failed authentication')
    return false
  }

  logs.log('ws client connected [ authentication ENABLED ]')
  return true
}
