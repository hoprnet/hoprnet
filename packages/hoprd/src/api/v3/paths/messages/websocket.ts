import type { Operation } from 'express-openapi'
import { WS_DEFAULT_RESPONSES, generateWsApiDescription } from '../../utils.js'

const GET: Operation = [
  async (_, res, _next) => {
    return res.status(404).end('Not found.')
  }
]

// This endpoint only exists to document the websocket's behaviour.
GET.apiDoc = {
  description: generateWsApiDescription(
    `This is a websocket endpoint which exposes a subset of message functions.
Incoming messages from other nodes are sent to the websocket client.
A client may also send message by sending the following data:
  { cmd: "sendmsg", args: { peerId: "SOME_PEER_ID", path: [], hops: 1} }
The command arguments follow the same semantics as in the dedicated API endpoint for sending messages.
`,
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
          example: '104,101,108,108,111,32,119,111,114,108,100'
        }
      }
    },
    '401': {
      $ref: '#/components/responses/Unauthorized'
    },
    '403': {
      $ref: '#/components/responses/Forbidden'
    }
  }
}

export default { GET }
