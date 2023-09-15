import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../../../utils.js'
import { generate_channel_id, Address, stringToU8a } from '@hoprnet/hopr-utils'

const GET: Operation = [
  async (req, res, _next) => {
    const { source, destination } = req.params

    try {
      let channel_id = generate_channel_id(Address.deserialize(stringToU8a(source)), Address.deserialize(stringToU8a(destination))).to_hex()

      return res.status(204).send(channel_id)
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

GET.apiDoc = {
  description: 'Converts two native addresses into a channel_id',
  tags: ['Tickets'],
  operationId: 'channelId',
  responses: {
    '204': {
      description: 'ChannelId'
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

export default { GET }
