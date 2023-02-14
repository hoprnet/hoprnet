import { STATUS_CODES } from '../../utils.js'
import { createToken, storeToken } from '../../../token.js'

import type { Operation } from 'express-openapi'

const POST: Operation = [
  async (req, res, _next) => {
    const { node, token } = req.context
    const { description, capabilities } = req.body

    const newToken = await createToken(node.db, token, capabilities, description)
    await storeToken(node.db, newToken)
    res.status(201).send({ token: newToken.id })
  }
]

POST.apiDoc = {
  description:
    'Create a new authentication token based on the given information. The new token is returned as part of the response body and must be stored by the client. It cannot be read again in cleartext and is lost, if the client loses the token. An authentication has a lifetime. It can be unbound, meaning it will not expire. Or it has a limited lifetime after which it expires. The requested limited lifetime is requested by the client in seconds.',
  tags: ['Tokens'],
  operationId: 'tokensCreate',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          required: ['capabilities'],
          properties: {
            capabilities: {
              description: 'Capabilities attached to the created token.',
              type: 'array',
              format: 'tokenCapabilities',
              minItems: 1,
              items: {
                $ref: '#/components/schemas/TokenCapability'
              }
            },
            lifetime: {
              type: 'integer',
              minimum: 1,
              description: 'Lifetime of the token in seconds since creation. Defaults to unlimited lifetime.'
            },
            description: {
              type: 'string',
              description: 'Description associated with the token.'
            }
          },
          example: {
            description: 'my test token',
            lifetime: 360,
            capabilities: []
          }
        }
      }
    }
  },
  responses: {
    '201': {
      description: 'Token succesfully created.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              token: {
                type: 'string',
                example: 'MYtoken1223',
                description: 'The generated token which must be used when authenticating for API calls.'
              }
            }
          }
        }
      }
    },
    '400': {
      description: 'Problem with inputs.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: { status: `${STATUS_CODES.INVALID_TOKEN_LIFETIME} | ${STATUS_CODES.INVALID_TOKEN_CAPABILITIES}` }
        }
      }
    },
    '403': {
      description: 'Missing capability to access endpoint'
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
