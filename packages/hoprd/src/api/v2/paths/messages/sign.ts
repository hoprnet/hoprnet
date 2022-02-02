import { Operation } from 'express-openapi'
import { u8aToHex } from '@hoprnet/hopr-utils'
import { STATUS_CODES } from '../../'

export const POST: Operation = [
  async (req, res, _next) => {
    try {
      const signature = await req.context.node.signMessage(new TextEncoder().encode(req.body.message))
      res.status(200).send({ signature: u8aToHex(signature) })
    } catch (err) {
      res.status(422).json({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
    }
  }
]

POST.apiDoc = {
  description:
    'Signs a message given using the node’s private key. Prefixes messsage with “HOPR Signed Message: ” before signing.',
  tags: ['Messages'],
  operationId: 'messageGetSignature',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          properties: {
            message: {
              description: 'The message to be signed.',
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
            $ref: '#/components/schemas/Signature'
          }
        }
      }
    },
    '422': {
      description: 'Unknown failure.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: { type: 'string', example: STATUS_CODES.UNKNOWN_FAILURE },
              error: { type: 'string', example: 'Full error message.' }
            }
          },
          example: { status: STATUS_CODES.UNKNOWN_FAILURE, error: 'Full error message.' }
        }
      }
    }
  }
}
