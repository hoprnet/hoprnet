import type Hopr from '@hoprnet/hopr-core'
import { Commands } from './commands/index.js'
import bodyParser from 'body-parser'

export default function setupAPI(node: Hopr, logs: any, options: any) {
  const http = require('http')
  const service = require('restana')()
  service.use(bodyParser.text({ type: '*/*' }))

  service.get('/api/v1/version', (_, res) => res.send(node.getVersion()))
  service.get('/api/v1/address/eth', async (_, res) => res.send((await node.getEthereumAddress()).toHex()))
  service.get('/api/v1/address/hopr', async (_, res) => res.send(node.getId().toB58String()))

  const cmds = new Commands(node)
  service.post('/api/v1/command', async (req, res) => {
    await node.waitForRunning()
    logs.log('Node is running')
    if (!options.testNoAuthentication && options.apiToken !== undefined) {
      if (req.headers['x-auth-token'] !== options.apiToken) {
        logs.log('command rejected: authentication failed')
        res.send('authentication failed', 403)
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

  const hostname = options.restHost
  const port = options.restPort
  http
    .createServer(service)
    .listen(port, hostname, () => {
      logs.log(`Rest server on ${hostname} listening on port ${port}`)
    })
    .on('error', (err: any) => {
      console.log(`Failed to start REST API.`)
      console.log(err)
      process.exit(1)
    })
}
