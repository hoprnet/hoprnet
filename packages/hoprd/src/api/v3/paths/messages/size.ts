import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'

const GET: Operation = [
  async (req, res, _next) => {
    const tag: number = req.query.tag as unknown as number
    const size = await req.context.inbox.size(tag)
    return res.status(200).send({ size })
  }
]

GET.apiDoc = {
  description: 'Get size of filtered message inbox.',
  tags: ['Messages'],
  operationId: 'messagesGetSize',
  parameters: [
    {
      in: 'query',
      name: 'tag',
      description: 'Tag used to filter target messages.',
      required: true,
      schema: {
        $ref: '#/components/schemas/MessageTag'
      }
    }
  ],
  responses: {
    '200': {
      description: 'Returns the message inbox size filtered by the given tag.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              size: {
                type: 'integer',
                minimum: 0,
                example: 1011
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

export default { GET }
