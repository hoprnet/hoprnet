import type { Operation } from 'express-openapi'
import type { State, StateOps } from '../../../../types'
import PeerId from 'peer-id'
import { STATUS_CODES } from '../../'

/**
 * Sets an alias and assigns the PeerId to it.
 * Updates HOPRd's state.
 * @returns new state
 */
export const setAlias = (stateOps: StateOps, alias: string, peerId: string): State => {
  try {
    const state = stateOps.getState()
    state.aliases.set(alias, PeerId.createFromB58String(peerId))
    stateOps.setState(state)
    return state
  } catch {
    throw Error(STATUS_CODES.INVALID_PEERID)
  }
}

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
  return peerId.toB58String()
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { stateOps } = req.context
    const { alias } = req.query

    try {
      const peerId = getAlias(stateOps.getState(), alias as string)
      return res.status(200).send({ peerId })
    } catch (err) {
      if (err.message.includes(STATUS_CODES.PEERID_NOT_FOUND)) {
        return res.status(404).send({ status: STATUS_CODES.PEERID_NOT_FOUND })
      } else {
        return res.status(500).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
      }
    }
  }
]

GET.apiDoc = {
  description: 'Get the PeerId of an alias.',
  tags: ['account'],
  operationId: 'accountGetPeerId',
  parameters: [
    {
      name: 'alias',
      in: 'query',
      description: 'Alias we want to fetch PeerId for.',
      required: true,
      schema: {
        type: 'string',
        example: 'Alice'
      }
    }
  ],
  responses: {
    '200': {
      description: 'PeerId found.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/PeerId'
          }
        }
      }
    },
    '404': {
      description: 'No alias found for the peerId.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: STATUS_CODES.PEERID_NOT_FOUND }
        }
      }
    },
    '500': {
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

export const POST: Operation = [
  async (req, res, _next) => {
    const { stateOps } = req.context
    const { peerId, alias } = req.body

    try {
      setAlias(stateOps, alias, peerId)
      return res.status(200).send()
    } catch (err) {
      if (err.message.includes(STATUS_CODES.INVALID_PEERID)) {
        return res.status(400).send({ status: STATUS_CODES.INVALID_PEERID, error: err.message })
      } else {
        return res.status(500).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
      }
    }
  }
]

POST.apiDoc = {
  description: 'Alias an address with a more memorable name',
  tags: ['account'],
  operationId: 'setAlias',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          properties: {
            peerId: { type: 'string', description: 'PeerId that we want to set alias to.' },
            alias: { type: 'string', description: 'Alias that we want to attach to peerId.' }
          },
          example: {
            peerId: '16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12',
            alias: 'Alice'
          }
        }
      }
    }
  },
  responses: {
    '200': {
      description: 'Alias set succesfully'
    },
    '400': {
      description: 'Invalid peerId',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: {
            status: STATUS_CODES.INVALID_PEERID
          }
        }
      }
    },
    '500': {
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

export const DELETE: Operation = [
  async (req, res, _next) => {
    const { stateOps } = req.context
    const { alias } = req.body

    try {
      removeAlias(stateOps, alias)
      return res.status(200).send()
    } catch (err) {
      return res.status(500).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
    }
  }
]

DELETE.apiDoc = {
  description: 'Unassign an alias from a PeerId',
  tags: ['account'],
  operationId: 'removeAlias',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          properties: {
            alias: { type: 'string', description: 'Alias that we want to remove.' }
          },
          example: {
            alias: 'Alice'
          }
        }
      }
    }
  },
  responses: {
    '200': {
      description: 'Alias removed succesfully'
    },
    '500': {
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
