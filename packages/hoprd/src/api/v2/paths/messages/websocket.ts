import { Operation } from 'express-openapi'

export const GET: Operation = [
  async (_, res, _next) => {
    return res.status(200).send('')
  }
]

// TODO: document
GET.apiDoc = {
  description: 'For developer convenience, we will be documenting the websocket endpoint here.',
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
