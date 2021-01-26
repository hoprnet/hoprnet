import Hopr, { FULL_VERSION } from '@hoprnet/hopr-core'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { Commands } from './commands'
import bodyParser from 'body-parser'

export default function setupAPI(node: Hopr<HoprCoreConnector>, logs: any, options: any) {
  const http = require('http')
  const service = require('restana')()
  service.use(bodyParser.text({ type: '*/*' }))

  service.get('/api/v1/version', (_, res) => res.send(FULL_VERSION))
  service.get('/api/v1/address/eth', async (_, res) => res.send(await node.paymentChannels.hexAccountAddress()))
  service.get('/api/v1/address/hopr', async (_, res) => res.send(await node.getId().toB58String()))

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
      logs.log(`Rest server on ${hostname} listening on port ${port}`)
    })
    .on('error', (err: any) => {
      console.log(`Failed to start REST API.`)
      console.log(err)
      process.exit(1)
    })
}
