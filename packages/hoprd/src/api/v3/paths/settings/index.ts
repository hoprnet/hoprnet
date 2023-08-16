import type { Operation } from 'express-openapi'
import type { State, StateOps } from '../../../../types.js'
import { STATUS_CODES } from '../../utils.js'

export const getSettings = (state: State) => {
  return state.settings
}

const GET: Operation = [
  async (req, res, _next) => {
    const { stateOps }: { stateOps: StateOps } = req.context

    try {
      const settings = getSettings(stateOps.getState())
      return res.status(200).send(settings)
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

GET.apiDoc = {
  description: `Get all of the node's settings.`,
  tags: ['Settings'],
  operationId: 'settingsGetSettings',
  responses: {
    '200': {
      description: 'Settings fetched succesfully.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/Settings'
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
