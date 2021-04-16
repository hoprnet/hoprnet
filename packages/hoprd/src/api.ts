import type Hopr from '@hoprnet/hopr-core'
import { Commands } from './commands'
import bodyParser from 'body-parser'
import { Logger } from '@hoprnet/hopr-utils'

const log = Logger.getLogger('hoprd.api')

export default function setupAPI(node: Hopr, logAdmin: any, options: any) {
  const http = require('http')
  const service = require('restana')()
  service.use(bodyParser.text({ type: '*/*' }))

  service.get('/api/v1/version', (_, res) => res.send(node.getVersion()))
  service.get('/api/v1/address/eth', async (_, res) => res.send((await node.getEthereumAddress()).toHex()))
  service.get('/api/v1/address/hopr', async (_, res) => res.send(node.getId().toB58String()))

  const cmds = new Commands(node)
  service.post('/api/v1/command', async (req, res) => {
    cmds.execute(req.body).then((resp: any) => {
      res.send(resp)
    })
  })

  const hostname = options.restHost
  const port = options.restPort
  http
    .createServer(service)
    .listen(port, hostname, () => {
      logAdmin.log(`Rest server on ${hostname} listening on port ${port}`)
    })
    .on('error', (err: any) => {
      log.error(`Failed to start REST API`, err)
      process.exit(1)
    })
}
