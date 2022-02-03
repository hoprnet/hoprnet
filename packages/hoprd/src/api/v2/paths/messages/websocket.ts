import { Operation } from 'express-openapi'

export const GET: Operation = [
  async (_, res, _next) => {
    return res.status(200).send('Connect to this endpoint with websocket client.')
  }
]

GET.apiDoc = {
  description:
    'This is a websocket endpoint to connect to via websocket client to receive realtime stream of messages sent to this node from other nodes on the network.',
  tags: ['Messages'],
  operationId: 'messagesWebsocket',
  responses: {
    '200': {
      description: '',
      content: {
        'application/json': {
          schema: {
            type: 'string',
            example: 'Connect to this endpoint with websocket client.'
          }
        }
      }
    }
  }
}
