import { peerIdFromString } from '@libp2p/peer-id'

import { STATUS_CODES } from '../../utils.js'

import type { Operation } from 'express-openapi'

const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context

    try {
      const aliases: Map<string, string> = await node.getAliases()
      return res.status(200).send(aliases)
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

GET.apiDoc = {
  description: 'Get all aliases you set previously and thier corresponding peer IDs.',
  tags: ['Aliases'],
  operationId: 'aliasesGetAliases',
  responses: {
    '200': {
      description: 'Returns List of Aliases and corresponding peerIds.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              alice: {
                $ref: '#/components/schemas/HoprAddress'
              },
              bob: {
                $ref: '#/components/schemas/HoprAddress'
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

const POST: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { peerId, alias } = req.body

    try {
      await node.setAlias(alias, peerIdFromString(peerId).toString())
      return res.status(201).send()
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

POST.apiDoc = {
  description:
    'Instead of using HOPR address, we can assign HOPR address to a specific name called alias. Give an address a more memorable alias and use it instead of Hopr address. Aliases are kept locally and are not saved or shared on the network.',
  tags: ['Aliases'],
  operationId: 'aliasesSetAlias',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          required: ['peerId', 'alias'],
          properties: {
            peerId: { format: 'peerid', type: 'string', description: 'PeerId that we want to set alias to.' },
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
    '201': {
      description: 'Alias set succesfully'
    },
    '400': {
      description: 'Invalid peerId. The format or length of the peerId is incorrect.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: {
            status: STATUS_CODES.INVALID_PEERID
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

export default { GET, POST }
