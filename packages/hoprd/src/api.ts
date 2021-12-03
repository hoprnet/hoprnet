import express from 'express'
import http from 'http'

import type Hopr from '@hoprnet/hopr-core'

import type { LogStream } from './logs'
import setupApiV1 from './api/v1'
import setupApiV2 from './api/v2'

export default function setupAPI(node: Hopr, logs: LogStream, options: { restPort: number; restHost: string }) {
  const hostname = options.restHost
  const port = options.restPort
  const service = express()

  setupApiV1(service, '/api/v1', node, logs, options)
  setupApiV2(service, '/api/v2', node, logs, options)

  http
    .createServer(service)
    .listen(port, hostname, () => {
      logs.log(`Rest API server on ${hostname} listening on port ${port}`)
    })
    .on('error', (err: any) => {
      logs.log(`Failed to start Rest API server: ${err}`)

      // bail out, fail hard because we cannot proceed with the overall
      // startup
      throw err
    })
}
