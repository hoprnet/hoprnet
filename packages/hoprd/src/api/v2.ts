import process from 'process'
import path from 'path'
import express from 'express'
import cors from 'cors'
import swaggerUi from 'swagger-ui-express'
import bodyParser from 'body-parser'
import { initialize } from 'express-openapi'
import PeerId from 'peer-id'

import type { Application, Request } from 'express'
import type Hopr from '@hoprnet/hopr-core'

import type { LogStream } from './../logs'

// The Rest API v2 is uses JSON for input and output, is validated through a
// Swagger schema which is also accessible for testing at:
// http://localhost:3001/api/v2/_swagger
export default function setupApiV2(service: Application, urlPath: string, node: Hopr, logs: LogStream, options: any) {
  // this API uses JSON data only
  service.use(urlPath, bodyParser.json())

  // enable all CORS requests
  service.use(urlPath, cors())

  // assign internal objects to each requests so they can be accessed within
  // handlers
  service.use(urlPath, (req, _res, next) => {
    req.context = new Context(node, logs)
    next()
  })
  // because express-openapi uses relative paths we need to figure out where
  // we are exactly
  const cwd = process.cwd()
  const packagePath = path.dirname(require.resolve('@hoprnet/hoprd/package.json'))
  const relPath = path.relative(cwd, packagePath)

  // useful documentation for the configuration of express-openapi can be found at:
  // https://github.com/kogosoftwarellc/open-api/tree/master/packages/express-openapi
  const apiInstance = initialize({
    app: service,
    // the spec resides in the package top-level folder
    apiDoc: path.join(relPath, 'rest-api-v2-spec.yaml'),
    // path to generated HTTP operations
    paths: path.join(relPath, 'lib/api/v2/paths'),
    // since we pass the spec directly we don't need to expose it via HTTP
    exposeApiDocs: false,
    // we use custom formats for particular internal data types
    customFormats: {
      peerId: (input) => {
        try {
          // this call will throw if the input is no peer id
          return !!PeerId.createFromB58String(input)
        } catch (_err) {
          return false
        }
      }
    },
    securityHandlers: {
      // TODO: We assume the handlers are always called in order. This isn't a
      // given and might change in the future. Thus, they should be made order-erindependent.
      keyScheme: function (req: Request, _scopes, _securityDefinition) {
        const apiToken = req.get('x-auth-token') || ''

        if (!options.testNoAuthentication && options.apiToken !== undefined && apiToken !== options.apiToken) {
          // because this is not the last auth check, we just indicate that
          // the authentication failed so the auth chain can continue
          return false
        }

        // successfully authenticated, will stop the auth chain and proceed with the request
        return true
      },
      passwordScheme: function (req: Request, _scopes, _securityDefinition) {
        const authEncoded = (req.get('authorization') || '').replace('Basic ', '')
        // we only expect a single value here, instead of the usual user:password
        const [apiToken, ..._rest] = decodeURI(Buffer.from(authEncoded, 'base64').toString('binary')).split(':')

        if (!options.testNoAuthentication && options.apiToken !== undefined && apiToken !== options.apiToken) {
          // because this is the last auth check, we must throw the appropriate
          // error to be sent back to the user
          throw {
            status: 403,
            challenge: 'Basic realm=hoprd',
            message: 'You must authenticate to access hoprd.'
          }
        }

        // successfully authenticated
        return true
      }
    }
  })

  // hook up the Swagger UI for our API spec
  // also see https://github.com/scottie1984/swagger-ui-express
  service.use(urlPath + '/_swagger', swaggerUi.serve)
  service.get(urlPath + '/_swagger', swaggerUi.setup(apiInstance.apiDoc, {}))

  service.use(urlPath, ((err, _req, res, _next) => {
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
