import express from 'express'
import bodyParser from 'body-parser'

import type Hopr from '@hoprnet/hopr-core'

import type { LogStream } from './../logs'
import { Commands } from './../commands'

export default function setupApiV1(service: Application, path: string, node: Hopr, logs: LogStream, options: any) {
  const router = express.Router()

  router.use(bodyParser.text({ type: '*/*' }))

  router.get('/version', (_, res) => res.send(node.getVersion()))
  router.get('/address/eth', async (_, res) => res.send((await node.getEthereumAddress()).toHex()))
  router.get('/address/hopr', async (_, res) => res.send(node.getId().toB58String()))

  const cmds = new Commands(node)
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

  service.use(path, router)
}
