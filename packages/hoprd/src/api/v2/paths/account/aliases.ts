import { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../'
import { State } from '../../../../types'

/**
 * @returns All PeerIds keyed by their aliases.
 */
export const getAliases = (state: Readonly<State>): { [alias: string]: string } => {
  return Array.from(state.aliases).reduce((result, [alias, peerId]) => {
    result[alias] = peerId.toB58String()
    return result
  }, {})
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { stateOps } = req.context

    try {
      const aliases = getAliases(stateOps.getState())
      return res.status(200).send(aliases)
    } catch (err) {
      return res.status(500).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
    }
  }
]

GET.apiDoc = {
  description: 'Get All aliases and thier corresponding PeerId.',
  tags: ['account'],
  operationId: 'accountGetPeerId',
  responses: {
    '200': {
      description: 'Returns List of Aliases and corresponding peerIds.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              Alice: { type: 'string', example: '16Uiu2HAmVfV4GKQhdECMqYmUMGLy84RjTJQxTWDcmUX5847roBar' },
              randomAlias: { type: 'string', example: '16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12' }
            }
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
