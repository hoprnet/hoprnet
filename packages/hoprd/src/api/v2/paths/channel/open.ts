import { Operation } from 'express-openapi'
import { isError } from '../../../../commands/v2'
import { setAlias } from '../../../../commands/v2/logic/alias'

export const parameters = []

export const POST: Operation = [
  async (req, res, _next) => {
    const { commands } = req.context
    const { peerId, alias } = req.body

    // NOTE: probably express can or already is handling it automatically
    if (!peerId || !alias) {
      return res.status(400).send({ status: 'missingBodyfields' })
    }

    const aliases = setAlias({ alias, peerId, state: commands.state })
    if (isError(aliases)) {
      return res.status(404).send({ status: 'invalidPeerId' })
    } else {
      return res.status(200).send({ status: 'success', aliases })
    }
  }
]

POST.apiDoc = {
  description: 'Opens a payment channel between you and the counter party provided',
  tags: ['channel'],
  operationId: 'openChannel',
  parameters: [],
  responses: {
    '200': {
      description: 'Channel succesfuly opened',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse',
            additionalProperties: {
              properties: {
                channelId: { type: 'string', example: '7b379578588920ca78fbf' }
              }
            }
          }
        }
      }
    },
    '400': {
      description: 'Problem with inputs',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: 'invalidPeerId | invalidAmountToFund' }
        }
      }
    },
    '500': {
      description: 'Insufficient balance to open channel',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse',
            additionalProperties: {
              properties: {
                tokensRequired: { type: 'string', example: '10' },
                currentBalance: { type: 'string', example: '9' }
              }
            }
          }
        }
      }
    }
  }
}
