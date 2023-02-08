import type { Operation } from 'express-openapi'
import { u8aToHex } from '@hoprnet/hopr-utils'
import { STATUS_CODES } from '../../utils.js'

const POST: Operation = [
  async (req, res, _next) => {
    try {
      const signature = req.context.node.signMessage(new TextEncoder().encode(req.body.message))
      return res.status(200).send({ signature: u8aToHex(signature) })
    } catch (err) {
      return res
        .status(422)
        .json({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

POST.apiDoc = {
  description:
    'Signs a message given using the node’s private key. Prefixes messsage with “HOPR Signed Message: ” before signing.',
  tags: ['Messages'],
  operationId: 'messagesSign',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          required: ['message'],
          properties: {
            message: {
              description: 'The message to be signed.',
              type: 'string'
            }
          }
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
            type: 'object',
            properties: {
              signature: {
                $ref: '#/components/schemas/Signature'
              }
            }
          }
        }
      }
    },
    '401': {
      $ref: '#/components/responses/Unauthorized'
    },
    '403': {
      $ref: '#/components/responses/Forbidden'
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

export default { POST }
