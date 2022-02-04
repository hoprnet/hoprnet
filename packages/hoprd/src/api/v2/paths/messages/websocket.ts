import { Operation } from 'express-openapi'

export const GET: Operation = [
  async (_, res, _next) => {
    return res.status(200).send('')
  }
]

// TODO: document
GET.apiDoc = {
  description:
    'For developer convenience, we will be documenting the websocket endpoint here, however, the websocket endpoint lives on a different port, depending on the configuration of your node the port might be different.',
  tags: ['Messages'],
  operationId: 'messagesWebsocket',
  responses: {
    '200': {
      description: '',
      content: {
        'application/json': {
          schema: {
            type: 'string',
            example: ''
          }
        }
      }
    }
  }
}
