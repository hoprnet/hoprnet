import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'

const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { alias } = req.params

    try {
      const peerId = await node.getAlias(alias as string)
      if (!peerId) throw Error(STATUS_CODES.PEERID_NOT_FOUND)

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
    const { node } = req.context
    const { alias } = req.params

    try {
      await node.removeAlias(alias)
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
