import { Operation } from 'express-openapi'
import { u8aToHex } from '@hoprnet/hopr-utils'

export const parameters = []

export const POST: Operation = [
  async (req, res, _next) => {
    try {
      const signedMessage = await req.context.node.signMessage(req.body.message)
      res.status(200).send({ signedMessage: u8aToHex(signedMessage) })
    } catch (err) {
      res.status(422).json({ error: err.message })
    }
  }
]

POST.apiDoc = {
  description:
    'Signs a message given using the node’s private key. Prefixes messsage with “HOPR Signed Message: ” before signing',
  tags: ['message'],
  operationId: 'messageGetSignature',
  parameters: [],
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          properties: {
            message: {
              description: 'The message which should be sent.',
              type: 'string'
            }
          },
          required: ['message']
        }
      }
    }
  },
  responses: {
    '200': {
      description: 'The message was signed successfully.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/SignedMessage'
          }
        }
      }
    }
  }
}
