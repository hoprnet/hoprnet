import { Operation } from 'express-openapi'

export const GET: Operation = [
  async (_, res, _next) => {
    return res.status(101).end()
  }
]

// This endpoint only exists to document the websocket behaviour.
GET.apiDoc = {
  description:
    'This is a websocket endpoint which streams incoming messages. Authentication (if enabled) is done via either passing an `apiToken` parameter in the url or cookie `X-Auth-Token`. Connect to the endpoint by using a WS client. No preview available. Example: `ws://127.0.0.1:3001/api/v2/messages/websocket/?apiToken=myApiToken`',
  tags: ['Messages'],
  operationId: 'messagesWebsocket',
  responses: {
    '101': {
      description: 'Switcing protocols'
    },
    '401': {
      description: 'Unauthorized'
    },
    '404': {
      description: 'Not found'
    }
  }
}
