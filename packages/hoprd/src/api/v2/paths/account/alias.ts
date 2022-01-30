import type { State } from '../../../../types'
import PeerId from 'peer-id'
import { Operation } from 'express-openapi'

export const setAlias = ({ peerId, alias, state }: { peerId: string; alias: string; state: State }): State => {
  try {
    state.aliases.set(alias, PeerId.createFromB58String(peerId))
    return state
  } catch (error) {
    throw Error('invalidPeerId')
  }
}

export const getAlias = ({ state, peerId }: { state: State; peerId: string }): string[] => {
  // @TODO: perhaps unnecessary
  try {
    PeerId.createFromB58String(peerId)
  } catch (error) {
    throw Error('invalidPeerId')
  }

  const aliases = Array.from(state.aliases.entries())
    .filter(([_, peerIdInMap]) => peerIdInMap.toB58String() === peerId)
    .map(([alias, _]) => alias)

  if (aliases.length === 0) {
    throw Error('aliasNotFound')
  }

  return aliases
}

export const parameters = []

export const GET: Operation = [
  async (req, res, _next) => {
    const { stateOps } = req.context
    const { peerId } = req.query

    if (!peerId) {
      return res.status(400).send({ status: 'noPeerIdProvided' })
    }

    try {
      const aliases = getAlias({ peerId: peerId as string, state: stateOps.getState() })
      return res.status(200).send({ status: 'success', aliases })
    } catch (err) {
      if (err.message.includes('invalidPeerId')) {
        return res.status(400).send({ error: err.message })
      } else {
        return res.status(404).send({ error: err.message })
      }
    }
  }
]

GET.apiDoc = {
  description: 'Get the alias/es assigned to a given address',
  tags: ['account'],
  operationId: 'getAlias',
  parameters: [
    {
      name: 'peerId',
      in: 'query',
      description: 'PeerId that we want to fetch aliases for',
      required: true,
      schema: {
        type: 'string',
        example: '16Uiu2HAmRFjDov6sbcZeppbnNFFTdx5hFoBzr8csBgevtKUex8y9'
      }
    }
  ],
  responses: {
    '200': {
      description: 'Alias/es fetched succesfully',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: { type: 'string', example: 'success' },
              aliases: {
                type: 'array',
                items: { type: 'string' },
                description: 'Aliases for given peerId',
                example: ['alias1']
              }
            }
          }
        }
      }
    },
    '400': {
      description: 'Invalid input',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: 'noPeerIdProvided | invalidPeerId' }
        }
      }
    },
    '404': {
      description: 'No alias found for the peerId',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: 'aliasNotFound' }
        }
      }
    }
  }
}

export const POST: Operation = [
  async (req, res, _next) => {
    const { stateOps } = req.context
    const { peerId, alias } = req.body

    // @TODO: probably express can or already is handling it automatically
    if (!peerId || !alias) {
      return res.status(400).send({ status: 'missingBodyfields' })
    }

    try {
      const aliases = setAlias({ alias, peerId, state: stateOps.getState() })
      return res.status(200).send({ status: 'success', aliases })
    } catch (err) {
      return res.status(400).send({ status: 'invalidPeerId' })
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
            peerId: '0x2C505741584f8591e261e59160D0AED5F74Dc29b',
            alias: 'john'
          }
        }
      }
    }
  },
  responses: {
    '200': {
      description: 'Alias set succesfully',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          }
        }
      }
    },
    '400': {
      description: 'Invalid peerId',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: {
            status: 'invalidPeerId | missingBodyfields'
          }
        }
      }
    }
  }
}
