import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'

const GET: Operation = [
  (req, res, _next) => {
    try {
      const version = req.context.node.getVersion()
      return res.status(200).json(version)
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

GET.apiDoc = {
  description: 'Get release version of the running node.',
  tags: ['Node'],
  operationId: 'nodeGetVersion',
  responses: {
    '200': {
      description: 'Returns the release version of the running node.',
      content: {
        'application/json': {
          schema: {
            type: 'string',
            description: 'Node version.',
            example: '1.83.5'
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
