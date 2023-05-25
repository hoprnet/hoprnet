import express from 'express'
import http from 'http'
import cors from 'cors'
import type Hopr from '@hoprnet/hopr-core'

import type { LogStream } from './logs.js'

export default function setupHealthcheck(node: Hopr, logs: LogStream, host: string, port: number) {
  const service = express()
  service.use(cors())
  service.get('/healthcheck/v1/version', (_, res) => res.send(node.getVersion()))
  service.get('/healthcheck/v1/network', (_, res) => res.send(node.network.id))

  http
    .createServer(service)
    .listen(port, host, () => {
      logs.log(`Healthcheck server on ${host} listening on port ${port}`)
    })
    .on('error', (err: any) => {
      logs.log(`Failed to start Healthcheck server: ${err}`)

      // bail out, fail hard because we cannot proceed with the overall
      // startup
      throw err
    })
}
