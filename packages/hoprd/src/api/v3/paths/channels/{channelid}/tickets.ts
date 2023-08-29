import { Hash, stringToU8a, debug } from '@hoprnet/hopr-utils'

import { STATUS_CODES } from '../../../utils.js'
import { formatTicket } from '../../tickets/index.js'

import type { Operation } from 'express-openapi'
import type { Hopr } from '@hoprnet/hopr-core'

const log = debug('hoprd:api:v3:channel-tickets')

const GET: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context
    const { channelid } = req.params

    try {
      const channelIdHash = Hash.deserialize(stringToU8a(channelid))
      const tickets = await node.getTickets(channelIdHash)
      if (tickets.length <= 0) {
        return res.status(404).send({ status: STATUS_CODES.TICKETS_NOT_FOUND })
      }
      const formattedTickets = tickets.map(formatTicket)
      return res.status(200).send(formattedTickets)
    } catch (err) {
      log(`Error getting tickets for channel ${channelid}: ${err}`)
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

GET.apiDoc = {
  description: 'Get tickets earned by relaying data packets by your node for the particular channel.',
  tags: ['Channels'],
  operationId: 'channelsGetTickets',
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
    '200': {
      description: 'Tickets fetched successfully.',
      content: {
        'application/json': {
          schema: {
            type: 'array',
            items: {
              $ref: '#/components/schemas/Ticket'
            }
          }
        }
      }
    },
    '400': {
      description: 'Invalid peerId.',
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

export default { GET }
