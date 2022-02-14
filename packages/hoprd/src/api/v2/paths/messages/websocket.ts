import type { Operation } from 'express-openapi'
import { WS_DEFAULT_RESPONSES, generateWsApiDescription } from '../../utils'

export const GET: Operation = [
  async (_, res, _next) => {
    return res.status(404).end('Not found. This is a websocket path.')
  }
]

// This endpoint only exists to document the websocket's behaviour.
GET.apiDoc = {
  description: generateWsApiDescription(
    'This is a websocket endpoint which streams incoming messages from other nodes.',
    '/messages/websocket'
  ),
  tags: ['Messages'],
  operationId: 'messagesWebsocket',
  responses: {
    ...WS_DEFAULT_RESPONSES,
    '206': {
      description: 'Incoming data',
      content: {
        'application/text': {
          schema: {
            type: 'string'
          },
          example: 'This is a super secret message'
        }
      }
    }
  }
}
