import type { Operation } from 'express-openapi'
import type { State, StateOps } from '../../../../types.js'
import { STATUS_CODES } from '../../utils.js'

/**
 * Removes alias and it's assigned PeerId.
 * Updates HOPRd's state.
 * @returns new state
 */
export const removeAlias = (stateOps: StateOps, alias: string): State => {
  const state = stateOps.getState()
  state.aliases.delete(alias)
  stateOps.setState(state)
  return state
}

/**
 * @returns The PeerId associated with the alias.
 */
export const getAlias = (state: Readonly<State>, alias: string): string => {
  const peerId = state.aliases.get(alias)
  if (!peerId) throw Error(STATUS_CODES.PEERID_NOT_FOUND)
  return peerId.toString()
}

const GET: Operation = [
  async (req, res, _next) => {
    const { stateOps }: { stateOps: StateOps } = req.context
    const { alias } = req.params

    try {
      const peerId = getAlias(stateOps.getState(), alias as string)
      return res.status(200).send({ peerId })
    } catch (err) {
      const errString = err instanceof Error ? err.message : err?.toString?.() ?? 'Unknown error'

      if (errString.includes(STATUS_CODES.PEERID_NOT_FOUND)) {
        return res.status(404).send({ status: STATUS_CODES.PEERID_NOT_FOUND })
      } else {
        return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: errString })
      }
    }
  }
]

GET.apiDoc = {
  description: 'Get the PeerId (Hopr address) that have this alias assigned to it.',
  tags: ['Aliases'],
  operationId: 'aliasesGetAlias',
  parameters: [
    {
      name: 'alias',
      in: 'path',
      description: 'Alias that we previously assigned to some PeerId.',
      required: true,
      schema: {
        type: 'string',
        example: 'Alice'
      }
    }
  ],
  responses: {
    '200': {
      description: `HOPR address was found for the provided alias.`,
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              peerId: { $ref: '#/components/schemas/HoprAddress' }
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
    '404': {
      description: `This alias was not assigned to any PeerId before. You can get the list of all PeerId's and thier corresponding aliases using /aliases endpoint.`,
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: { status: STATUS_CODES.PEERID_NOT_FOUND }
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

const DELETE: Operation = [
  async (req, res, _next) => {
    const { stateOps } = req.context
    const { alias } = req.params

    try {
      removeAlias(stateOps, alias)
      return res.status(204).send()
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

DELETE.apiDoc = {
  description:
    'Unassign an alias from a PeerId. You can always assign back alias to that PeerId using /aliases endpoint.',
  tags: ['Aliases'],
  operationId: 'aliasesRemoveAlias',
  parameters: [
    {
      name: 'alias',
      in: 'path',
      description: 'Alias that we want to remove.',
      required: true,
      schema: {
        type: 'string',
        example: 'Alice'
      }
    }
  ],
  responses: {
    '204': {
      description: 'Alias removed succesfully.'
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

export default { GET, DELETE }
