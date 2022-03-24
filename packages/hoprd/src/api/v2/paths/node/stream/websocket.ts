import type { Operation } from 'express-openapi'
import { WS_DEFAULT_RESPONSES, generateWsApiDescription } from '../../../utils'

export const GET: Operation = [
  async (_, res, _next) => {
    return res.status(404).end('Not found.')
  }
]

// This endpoint only exists to document the websocket's behaviour.
GET.apiDoc = {
  description: generateWsApiDescription(
    'This is a websocket endpoint which streams legacy hopr-admin data excluding messages.',
    '/node/stream/websocket'
  ),
  tags: ['Node'],
  operationId: 'nodeStreamWebsocket',
  deprecated: true,
  responses: {
    ...WS_DEFAULT_RESPONSES,
    '206': {
      description: 'Incoming data.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              type: {
                type: 'string',
                description: 'Type of data',
                example: ['log', 'fatal-error', 'status', 'connected']
              },
              timestamp: {
                type: 'number',
                description: 'Timestamp in miliseconds',
                example: 1644587213977
              },
              content: {
                type: 'string',
                description: 'The text content',
                example: 'Opening channel...'
              }
            }
          },
          example: {
            type: 'log',
            timestamp: 1644587213977,
            content: 'Opening channel...'
          }
        }
      }
    }
  }
}
