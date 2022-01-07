import { Operation } from 'express-openapi'
import { u8aToHex } from '@hoprnet/hopr-utils'

export const parameters = []

export const POST: Operation = [
  async (req, res, _next) => {
    try {
      const signedMessage = await req.context.node.signMessage(req.body.body)
      res.status(200).send({ signedMessage: u8aToHex(signedMessage) })
    } catch (err) {
      res.status(422).json({ error: err.message })
    }
  }
]

POST.apiDoc = {
  description:
    'Signs a message given using the node’s private key. Prefixes messsage with “HOPR Signed Message: ” before signing',
  tags: ['sign'],
  operationId: 'signMessage',
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
            }
          },
          required: ['body']
        }
      }
    }
  },
  responses: {
    '200': {
      description: 'The message was signed successfully.'
    }
  }
}
