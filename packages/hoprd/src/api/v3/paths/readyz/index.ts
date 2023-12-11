import type { Hopr } from '@hoprnet/hopr-utils'
import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'

export const GET: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context

    try {
      if (node.isRunning()) {
        return res.status(200).send()
      } {
        return res.status(422)
          .send({ status: STATUS_CODES.APPLICATION_CHECK_FAILED, error: 'Not ready yet' })
      }
    } catch (err) { {}
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

GET.apiDoc = {
  description: 'Check whether the node is ready',
  tags: ['CheckReady'],
  operationId: 'CheckNodeReady',
  parameters: [],
  responses: {
    '200': {
      description: 'The node is ready',
      content: {
        'application/json': {
          schema: { }
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
