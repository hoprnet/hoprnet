import { v4 as uuidv4 } from 'uuid'
import { createHash } from 'crypto'

import { STATUS_CODES } from '../../utils.js'
import { storeToken, authenticateToken } from '../../../token.js'

import type { Operation } from 'express-openapi'
import type { HoprDB } from '@hoprnet/hopr-utils'

async function generateNewId(db: HoprDB): Promise<string> {
  let id = undefined

  // iterate until we find a usable id
  while (!id) {
    const uuid = uuidv4()
    const nextId = createHash('sha256').update(uuid).digest('base64')
    // try to load the token given the new id
    const token = await authenticateToken(db, nextId)
    if (!token) {
      // no token found, id can be used
      id = nextId
    }
  }

  return id
}

const POST: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { description, capabilities } = req.body

    const id = await generateNewId(node.db)

    const token = {
      id,
      description,
      capabilities
    }

    await storeToken(node.db, token)

    res.status(201).send({ token: id })
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
              format: 'capabilities',
              type: 'string',
              description: 'Capabilities attached to the created token.'
            },
            lifetime: {
              format: 'amount',
              type: 'integer',
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
