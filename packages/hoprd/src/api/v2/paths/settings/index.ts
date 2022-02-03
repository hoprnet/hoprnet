import type { Operation } from 'express-openapi'
import type { State } from '../../../../types'
import { STATUS_CODES } from '../../'

export const getSettings = (state: State) => {
  return state.settings
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { stateOps } = req.context

    try {
      const settings = getSettings(stateOps.getState())
      return res.status(200).send(settings)
    } catch (error) {
      return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: error.message })
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
            type: 'object',
            properties: {
              includeRecipient: {
                type: 'boolean',
                description: 'Prepends your address to all messages.',
                example: true
              },
              strategy: {
                type: 'string',
                enum: ['passive', 'promiscuous'],
                example: 'passive'
              }
            },
            description: "The node's settings."
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
