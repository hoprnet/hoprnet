import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../'
import { getSetting } from './settings/{setting}'

export const GET: Operation = [
  async (req, res, _next) => {
    const { stateOps, node } = req.context

    try {
      const settings = getSetting({
        node,
        state: stateOps.getState()
      })
      return res.status(200).send({ settings })
    } catch (error) {
      return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: error.message })
    }
  }
]

GET.apiDoc = {
  description: `Get all of this node's settings values.`,
  tags: ['Settings'],
  operationId: 'getSetting',
  responses: {
    '200': {
      description: 'Settings fetched succesfully.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              settings: {
                type: 'array',
                items: {
                  $ref: '#/components/schemas/Setting'
                },
                description: 'Setting/s fetched'
              }
            }
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
