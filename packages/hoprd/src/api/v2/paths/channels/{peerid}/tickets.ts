import type Hopr from '@hoprnet/hopr-core'
import type { Operation } from 'express-openapi'
import PeerId from 'peer-id'
import { STATUS_CODES } from '../../../'
import { formatTicket } from '../../tickets'

export const getTickets = async (node: Hopr, peerId: string) => {
  const tickets = await node.getTickets(PeerId.createFromB58String(peerId))
  return tickets.map(formatTicket)
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { peerid } = req.params

    try {
      const tickets = await getTickets(node, peerid)
      if (tickets.length <= 0) {
        return res.status(404).send({ status: STATUS_CODES.TICKETS_NOT_FOUND })
      }
      return res.status(200).send(tickets)
    } catch (err) {
      return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
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
      name: 'peerid',
      example: '16Uiu2HAm91QFjPepnwjuZWzK5pb5ZS8z8qxQRfKZJNXjkgGNUAit',
      required: true,
      schema: {
        type: 'string',
        description: 'PeerId attached to the channel.',
        example: '16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12'
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
