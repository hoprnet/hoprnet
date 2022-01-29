import { Operation } from 'express-openapi'
import { isError } from '../../logic'
import { listOpenChannels } from '../../logic/channel'

export const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context

    const channels = await listOpenChannels({ node })
    if (isError(channels)) {
      return res.status(500).send({ status: channels.message })
    } else {
      return res.status(200).send({ status: 'success', channels })
    }
  }
]

GET.apiDoc = {
  description: 'Lists your currently open channels',
  tags: ['channel'],
  operationId: 'listOpenChannels',
  parameters: [],
  responses: {
    '200': {
      description: 'Channels fetched succesfully',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: { type: 'string', example: 'success' },
              channels: {
                type: 'object',
                properties: {
                  incoming: { type: 'array', items: { $ref: '#/components/schemas/Channel' } },
                  outgoing: { type: 'array', items: { $ref: '#/components/schemas/Channel' } }
                }
              }
            }
          }
        }
      }
    },
    '500': {
      description: 'No alias found for the peerId',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: 'failure' }
        }
      }
    }
  }
}
