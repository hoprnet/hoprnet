import { Operation } from 'express-openapi'
import { isError } from '../../logic'
import { getAlias, setAlias } from '../../logic/alias'

export const parameters = []

export const GET: Operation = [
  async (req, res, _next) => {
    const { state } = req.context
    const { peerId } = req.query

    if (!peerId) {
      return res.status(400).send({ status: 'noPeerIdProvided' })
    }

    const aliases = getAlias({ peerId: peerId as string, state })
    if (isError(aliases)) {
      return res.status(aliases.message === 'invalidPeerId' ? 400 : 404).send({ status: aliases.message })
    } else {
      return res.status(200).send({ status: 'success', aliases })
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
        example: 'examplePeerId'
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
    const { state } = req.context
    const { peerId, alias } = req.body

    // NOTE: probably express can or already is handling it automatically
    if (!peerId || !alias) {
      return res.status(400).send({ status: 'missingBodyfields' })
    }

    const aliases = setAlias({ alias, peerId, state: state })
    if (isError(aliases)) {
      return res.status(400).send({ status: 'invalidPeerId' })
    } else {
      return res.status(200).send({ status: 'success', aliases })
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
