import type { Operation } from 'express-openapi'
import { WS_DEFAULT_RESPONSES, generateWsApiDescription } from '../../../utils'

export const GET: Operation = [
  async (_, res, _next) => {
    return res.status(404).end('Not found. This is a websocket path.')
  }
]

// This endpoint only exists to document the websocket's behaviour.
GET.apiDoc = {
  description: generateWsApiDescription(
    'This is a websocket endpoint which streams legacy hopr-admin logs.',
    '/node/logs/websocket'
  ),
  tags: ['Node'],
  operationId: 'nodeLogsWebsocket',
  deprecated: true,
  responses: {
    ...WS_DEFAULT_RESPONSES,
    '206': {
      description: 'Incoming data',
      content: {
        'application/text': {
          schema: {
            type: 'object',
            properties: {
              ts: {
                type: 'number',
                description: 'Timestamp in miliseconds'
              },
              content: {
                type: 'string',
                description: 'The log message'
              }
            }
          },
          example: JSON.stringify(
            {
              ts: 1644587213977,
              content: 'Opening channel...'
            },
            null,
            2
          )
        }
      }
    }
  }
}
