import express from 'express'
import swaggerUi from 'swagger-ui-express'
import bodyParser from 'body-parser'
import { initialize } from 'express-openapi'

import type Hopr from '@hoprnet/hopr-core'

import type { LogStream } from './../logs'

// The Rest API v2 is uses JSON for input and output, is validated through a
// Swagger schema which is also accessible for testing at:
// http://localhost:3001/api/v2/_swagger
export default function setupApiV2(service: Application, path: string, node: Hopr, logs: LogStream, _options: any) {
  // this API uses JSON data only
  service.use(path, bodyParser.json())

  // assign internal objects to each requests so they can be accessed within
  // handlers
  service.use(path, (req, _res, next) => {
    req.context = new Context(node, logs)
    next()
  })

  // useful documentation for the configuration of express-openapi can be found at:
  // https://github.com/kogosoftwarellc/open-api/tree/master/packages/express-openapi
  const apiInstance = initialize({
    app: service,
    // the spec resides in the package top-level folder
    apiDoc: 'rest-api-v2-spec.yaml',
    // path to generated HTTP operations
    paths: './lib/api/v2/paths',
    // since we pass the spec directly we don't need to expose it via HTTP
    exposeApiDocs: false
  })

  // hook up the Swagger UI for our API spec
  // also see https://github.com/scottie1984/swagger-ui-express
  service.use(path + '/_swagger', swaggerUi.serve)
  service.get(path + '/_swagger', swaggerUi.setup(apiInstance.apiDoc, {}))

  service.use(path, ((err, _req, res, _next) => {
    res.status(err.status).json(err)
  }) as express.ErrorRequestHandler)
}

// In order to pass custom objects along with each request we build a context
// which is attached during request processing.
export class Context {
  constructor(public node: Hopr, public logs: LogStream) {}
}

declare global {
  namespace Express {
    interface Request {
      context: Context
    }
  }
}
