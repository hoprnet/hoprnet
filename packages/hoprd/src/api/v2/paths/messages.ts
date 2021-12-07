import { Operation } from 'express-openapi'
import { encode } from 'rlp'
import PeerId from 'peer-id'

export const parameters = []

export const POST: Operation = [
  async (req, res, _next) => {
    const message = encode([req.body.body])
    let path = req.body.path
    const recipient: PeerId = path.pop()

    await req.context.node.sendMessage(message, recipient, path)

    res.status(204).send()
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
            path: {
              description:
                'The path is ordered list of peer ids through which the message should be sent. ' +
                'The last path element is the receiver of the message.',
              type: 'array',
              items: {
                description: 'A valid HOPR peer id',
                type: 'string',
                format: 'peerId',
                minItems: 1,
                maxItems: 3
              }
            }
          },
          required: ['body', 'path']
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
