import process from 'process'
import path from 'path'
import fs from 'fs'
import express from 'express'
import cors from 'cors'
import swaggerUi from 'swagger-ui-express'
import bodyParser from 'body-parser'
import { initialize } from 'express-openapi'
import { peerIdFromString } from '@libp2p/peer-id'
import BN from 'bn.js'

import { Hash, debug, stringToU8a, Address, OffchainPublicKey } from '@hoprnet/hopr-utils'
import {
  authenticateWsConnection,
  getStatusCodeForInvalidInputInRequest,
  removeQueryParams,
  encodeMessage
} from './utils.js'
import { authenticateToken, authorizeToken, validateTokenCapabilities } from './token.js'
import { STATUS_CODES } from './v3/utils.js'

import type { Server } from 'http'
import type { Application, Request } from 'express'
import type { WebSocket, WebSocketServer } from 'ws'
import type { Hopr } from '@hoprnet/hopr-core'
import { SettingKey, type StateOps } from '../types.js'
import type { Token } from './token.js'
import type { Database, ApplicationData, MessageInbox } from '../../../hoprd/lib/hoprd_hoprd.js'

const debugLog = debug('hoprd:api:v3')

enum AuthResult {
  Failed,
  Authenticated,
  Authorized
}

async function handleWebsocketMessage(node: Hopr, data: string) {
  const msg = JSON.parse(data)
  const cmd = msg['cmd']
  const args = msg['args']

  // handling custom websocket protocol
  switch (cmd) {
    case 'sendmsg':
      const body = encodeMessage(args['body'])
      const recipient = peerIdFromString(args['peerId'])
      const hops = args['hops']

      // only set path if given, otherwise a path will be chosen by hopr core
      let path: OffchainPublicKey[]
      if (args['path'] != undefined) {
        path = args['path'].map((peer: string) => OffchainPublicKey.from_peerid_str(peer))
      }

      // send message and return ack challenge over websocket
      let ackChallenge = await node.sendMessage(body, recipient, path, hops)
      node.emit('hopr:message-ack-challenge', ackChallenge)

      break
    default:
      debugLog(`Ignoring websocket message with command ${cmd}`)
  }
}

async function authenticateAndAuthorize(
  db: Database,
  req: Request,
  reqToken: string,
  superuserToken: string
): Promise<AuthResult> {
  // 1. check superuser token
  const isSuperuserAuthenticated = reqToken === superuserToken

  // continue early if superuser is authenticated, no authorization checks needed
  if (isSuperuserAuthenticated) {
    return AuthResult.Authorized
  }

  // 2. check user token authentication
  reqToken = decodeURIComponent(reqToken)
  const token: Token = await authenticateToken(db, reqToken)
  if (token) {
    // 3. token was found, therefore is authenticated, next check authorization
    const endpointRef: string = req['operationDoc'].operationId
    if (await authorizeToken(db, token, endpointRef)) {
      req.context.token = token
      return AuthResult.Authorized
    }
    return AuthResult.Authenticated
  }

  return AuthResult.Failed
}

export const RPCH_MESSAGE_REGEXP = /^(\d+)\|\d+\|\d+\|[\w\W]+$/

// The Rest API v3 is uses JSON for input and output, is validated through a
// Swagger schema which is also accessible for testing at:
// http://localhost:3001/api/v3/_swagger
export async function setupRestApi(
  service: Application,
  urlPath: string,
  node: Hopr,
  inbox: MessageInbox,
  stateOps: StateOps,
  options: {
    apiToken?: string
    disableApiAuthentication?: boolean
  }
): Promise<ReturnType<typeof initialize>> {
  // this API uses JSON data only
  service.use(urlPath, bodyParser.json())

  // enable all CORS requests
  service.use(urlPath, cors())

  // assign internal objects to each requests so they can be accessed within
  // handlers
  service.use(
    urlPath,
    function addNodeContext(req, _res, next) {
      req.context = { node, inbox, stateOps }
      next()
    }
      // Need to explicitly bind the instances to the function
      // to make sure the right instances are present
      .bind({ node, inbox, stateOps })
  )
  // because express-openapi uses relative paths we need to figure out where
  // we are exactly
  const cwd = process.cwd()
  const packagePath = path.dirname(new URL('../../package.json', import.meta.url).pathname)
  const relPath = path.relative(cwd, packagePath)
  const apiBaseSpecPath = path.join(relPath, 'rest-api-v3-spec.yaml')
  const apiFullSpecPath = path.join(relPath, 'rest-api-v3-full-spec.json')
  const apiPathsPath = path.join(relPath, 'lib/api/v3/paths')
  const encodedApiToken = encodeURIComponent(options.apiToken)

  // useful documentation for the configuration of express-openapi can be found at:
  // https://github.com/kogosoftwarellc/open-api/tree/master/packages/express-openapi
  const apiInstance = await initialize({
    app: service,
    // the spec resides in the package top-level folder
    apiDoc: apiBaseSpecPath,
    // path to generated HTTP operations
    paths: apiPathsPath,
    pathsIgnore: /\.spec$/,
    routesGlob: '**/*.js',
    routesIndexFileRegExp: /(?:index)?\.js$/,
    // since we pass the spec directly we don't need to expose it via HTTP
    exposeApiDocs: false,
    errorMiddleware: function (err, req, res, next) {
      // @fixme index-0 access does not always work
      if (err.status === 400) {
        const path = String(err.errors[0].path) || ''
        res.status(err.status).send({ status: getStatusCodeForInvalidInputInRequest(path) })
        return
      }
      if (err.status === 401) {
        // distinguish between 401 and 403
        if (req.context.authResult === AuthResult.Failed) {
          res.status(401).send({ status: STATUS_CODES.UNAUTHORIZED, error: 'authentication failed' })
          return
        }
        if (req.context.authResult === AuthResult.Authenticated) {
          res.status(403).send({ status: STATUS_CODES.UNAUTHORIZED, error: 'authorization failed' })
          return
        }
      }
      next(err)
    },
    // we use custom formats for particular internal data types
    customFormats: {
      channelid: (input) => {
        try {
          // this call will throw if the input is no hash
          Hash.deserialize(stringToU8a(input))
        } catch (err) {
          return false
        }
        return true
      },
      peerid: (input) => {
        try {
          // this call will throw if the input is no peer id
          peerIdFromString(input)
        } catch (err) {
          return false
        }
        return true
      },
      ethereumaddress: (input) => {
        try {
          Address.from_string(input)
        } catch (err) {
          return false
        }
        return true
      },
      amount: (input) => {
        try {
          new BN(input)
        } catch (err) {
          return false
        }
        return true
      },
      settingKey: (input) => {
        return Object.values(SettingKey).includes(input)
      },
      tokenCapabilities: (input) => {
        return validateTokenCapabilities(input)
      }
    },
    securityHandlers: {
      // TODO: We assume the handlers are always called in order. This isn't a
      // given and might change in the future. Thus, they should be made order-independent.
      keyScheme: async function (req: Request, _scopes, _securityDefinition) {
        // skip checks if authentication is disabled
        if (options.disableApiAuthentication) return true

        // Applying multiple URI encoding is an identity
        let apiTokenFromUser = encodeURIComponent(req.get('x-auth-token') || '')

        req.context.authResult = await authenticateAndAuthorize(node.db, req, apiTokenFromUser, encodedApiToken)
        return req.context.authResult === AuthResult.Authorized
      }.bind({ options }),
      passwordScheme: async function (req: Request, _scopes, _securityDefinition) {
        // skip checks if authentication is disabled
        if (options.disableApiAuthentication) return true

        const authEncoded = (req.get('authorization') || '').replace('Basic ', '')
        // We only expect a single value here, instead of the usual user:password, so we take the user part as token
        const apiTokenFromUser = encodeURIComponent(Buffer.from(authEncoded, 'base64').toString('binary').split(':')[0])

        const result = await authenticateAndAuthorize(node.db, req, apiTokenFromUser, encodedApiToken)
        if (result === AuthResult.Authorized) {
          return true
        }
        // if authentication or authorization failed capture highest failure mode
        req.context.authResult = Math.max(result, req.context.authResult)
        return false
      }.bind({ options })
    }
  })

  // hook up the Swagger UI for our API spec
  // also see https://github.com/scottie1984/swagger-ui-express
  service.use(urlPath + '/_swagger', swaggerUi.serve)
  service.get(urlPath + '/_swagger', swaggerUi.setup(apiInstance.apiDoc, {}))

  // Write the api spec to disk for use outside of the server.
  // We only do this if CI or DEBUG are set to prevent this happening in
  // production environments.
  if (process.env.DEBUG || process.env.CI) {
    try {
      fs.writeFile(apiFullSpecPath, JSON.stringify(apiInstance.apiDoc), (err) => {
        if (err) {
          debugLog(`Error: Could not write full Rest API v3 spec file to ${apiFullSpecPath}: ${err}`)
          return
        }
        debugLog(`Written full Rest API v3 spec file to ${apiFullSpecPath}`)
      })
    } catch (err) {
      debugLog(`Error: Could not write full Rest API v3 spec file to ${apiFullSpecPath}: ${err}`)
    }
  }

  service.use(urlPath, ((err, _req, res, _next) => {
    res.status(err.status).json(err)
  }) as express.ErrorRequestHandler)

  return apiInstance
}

const WS_PATHS = {
  NONE: '', // used for testing
  MESSAGES: '/api/v3/messages/websocket'
}

export function setupWsApi(server: Server, wss: WebSocketServer, node: Hopr, options: { apiToken?: string }) {
  // before upgrade to WS, we perform various checks
  server.on('upgrade', function upgrade(req, socket, head) {
    debugLog('WS client attempt to upgrade')
    const path = removeQueryParams(req.url)
    const needsAuth = !!options.apiToken

    // check if path is supported
    if (!Object.values(WS_PATHS).includes(path)) {
      debugLog(`WS client path '${path}' does not exist`)
      socket.end('HTTP/1.1 404 Not Found\r\n\r\n', () => socket.destroy())
      return
    }

    // check if request is authenticated
    if (needsAuth && !authenticateWsConnection(req, options.apiToken)) {
      debugLog('WS client failed authentication')
      socket.end('HTTP/1.1 401 Unauthorized\r\n\r\n', () => socket.destroy())
      return
    }

    // log connection status
    if (!needsAuth) debugLog('WS client connected [ authentication DISABLED ]')
    else debugLog('WS client connected [ authentication ENABLED ]')

    // upgrade to WS protocol
    wss.handleUpgrade(req, socket, head, (socket_: WebSocket) => {
      wss.emit('connection', socket_, req)
    })
  })

  wss.on('connection', (socket: WebSocket, req) => {
    debugLog('WS client connected!')
    const path = removeQueryParams(req.url)

    // per-connection tracking of whether the connection is alive using
    // websocket-native ping/pong
    // ping messages are sent every 15 seconds
    let isAlive = true

    const heartbeatReceived = () => (isAlive = true)

    const heartbeatInterval = setInterval(() => {
      // terminate connection if the previous ping check failed
      if (isAlive === false) return socket.terminate()

      isAlive = false
      socket.ping()
    }, 15000)

    socket.on('error', (err: string) => {
      debugLog('WS error', err.toString())
    })

    // if we receive the pong back, the connection is marked as alive
    socket.on('pong', heartbeatReceived)

    socket.on('close', () => {
      clearInterval(heartbeatInterval)
    })

    socket.on('message', (data: string) => {
      // only MESSAGES path is supported
      if (path !== WS_PATHS.MESSAGES) return
      handleWebsocketMessage(node, data)
    })

    if (path === WS_PATHS.MESSAGES) {
      node.on('hopr:message', (data: ApplicationData) => {
        socket.send(data.plain_text.toString())
      })
      node.on('hopr:message-ack-challenge', (ackChallenge: string) => {
        socket.send(`ack-challenge:${ackChallenge}`)
      })
      node.on('hopr:message-acknowledged', (ackChallenge: string) => {
        socket.send(`ack:'${ackChallenge}'`)
      })
    } else {
      // close connection on unsupported paths
      socket.close(1000)
    }
  })
}

// In order to pass custom objects along with each request we build a context
// which is attached during request processing.
export class Context {
  public token: Token
  public authResult: AuthResult

  constructor(public node: Hopr, public inbox: MessageInbox, public stateOps: StateOps) {}
}

declare global {
  namespace Express {
    interface Request {
      context: {
        node: Hopr
        stateOps: StateOps
        token: Token
        authResult: AuthResult
        inbox: MessageInbox
      }
    }
  }
}
