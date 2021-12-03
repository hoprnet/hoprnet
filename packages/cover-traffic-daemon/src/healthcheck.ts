import express from 'express'
import http from 'http'

import { debug } from '@hoprnet/hopr-utils'

import type Hopr from '@hoprnet/hopr-core'

const log = debug('hopr:cover-traffic')

export default function setupHealthcheck(node: Hopr, host: string, port: number) {
  const service = express()

  service.get('/healthcheck/v1/version', (_, res) => res.send(`CT node: ${node.getVersion()}`))

  http
    .createServer(service)
    .listen(port, host, () => {
      log(`Healthcheck server on ${host} listening on port ${port}`)
    })
    .on('error', (err: any) => {
      log(`Failed to start Healthcheck server: ${err}`)

      // bail out, fail hard because we cannot proceed with the overall
      // startup
      throw err
    })
}
