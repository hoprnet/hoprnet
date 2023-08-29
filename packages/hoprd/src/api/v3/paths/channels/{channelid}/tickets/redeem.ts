import { Hash, stringToU8a } from '@hoprnet/hopr-utils'

import { STATUS_CODES } from '../../../../utils.js'

import type { Operation } from 'express-openapi'
import type { Hopr } from '@hoprnet/hopr-core'

const POST: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context
    const { channelid } = req.params

    try {
      const channelIdHash = Hash.deserialize(stringToU8a(channelid))
      const tickets = await node.getTickets(channelIdHash)
      if (tickets.length <= 0) {
        return res.status(404).send({ status: STATUS_CODES.TICKETS_NOT_FOUND })
      }
      await node.redeemTicketsInChannel(channelIdHash)
      return res.status(204).send()
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

POST.apiDoc = {
  description:
    'Redeems your tickets for this channel. Redeeming will change your tickets into Hopr tokens if they are winning ones. You can check how much tickets given channel has by calling /channels/{channelid}/tickets endpoint. Do this before channel is closed as neglected tickets are no longer valid for redeeming.',
  tags: ['Channels'],
  operationId: 'channelsRedeemTickets',
  parameters: [
    {
      in: 'path',
      name: 'channelid',
      required: true,
      schema: {
        format: 'channelid',
        type: 'string'
      }
    }
  ],
  responses: {
    '204': {
      description: 'Tickets redeemed successfully.'
    },
    '400': {
      description: 'Invalid channel id.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: {
            status: STATUS_CODES.INVALID_CHANNELID
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
      description:
        'Tickets were not found for that channel. That means that no messages were sent inside this channel yet.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: {
            status: STATUS_CODES.TICKETS_NOT_FOUND
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

export default { POST }
