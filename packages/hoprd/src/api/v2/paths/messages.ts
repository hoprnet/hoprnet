import { Operation } from 'express-openapi'
import { encode } from 'rlp'
import PeerId from 'peer-id'

export const parameters = []

export const POST: Operation = [
  async (req, res, _next) => {
    const message = encode([req.body.body])
    const path = req.body.path
    const recipient: PeerId = PeerId.createFromB58String(req.body.recipient)

    try {
      await req.context.node.sendMessage(message, recipient, path)
      res.status(204).send()
    } catch (err) {
      res.status(422).json({ error: err.message })
    }
  }
]

POST.apiDoc = {
  description: 'Send a message to another peer using a given path.',
  tags: ['messages'],
  operationId: 'messagesSend',
  parameters: [],
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          properties: {
            body: {
              description: 'The message body which should be sent.',
              type: 'string'
            },
            recipient: {
              description: 'The recipient HOPR peer id, to which the message is sent.',
              type: 'string',
              format: 'peerId'
            },
            path: {
              description: 'The path is ordered list of peer ids through which the message should be sent. ',
              type: 'array',
              default: [],
              items: {
                description: 'A valid HOPR peer id',
                type: 'string',
                format: 'peerId',
                minItems: 1,
                maxItems: 3
              }
            }
          },
          required: ['body', 'recipient']
        }
      }
    }
  },
  responses: {
    '204': {
      description: 'The message was sent successfully. NOTE: This does not imply successful delivery.'
    }
  }
}
