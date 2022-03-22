import type Hopr from '@hoprnet/hopr-core'
import type { Ticket } from '@hoprnet/hopr-utils'
import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils'

export const formatTicket = (ticket: Ticket) => {
  return {
    counterparty: ticket.counterparty.toHex(),
    challenge: ticket.challenge.toHex(),
    epoch: ticket.epoch.toBN().toString(),
    index: ticket.index.toBN().toString(),
    amount: ticket.amount.toBN().toString(),
    winProb: ticket.winProb.toBN().toString(),
    channelEpoch: ticket.channelEpoch.toBN().toString(),
    signature: ticket.signature.toHex()
  }
}

export const getAllTickets = async (node: Hopr) => {
  const tickets = await node.getAllTickets()
  return tickets.map(formatTicket)
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context

    try {
      const tickets = await getAllTickets(node)
      return res.status(200).send(tickets)
    } catch (err) {
      return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
    }
  }
]

// TODO: tickets missing param ???
GET.apiDoc = {
  description: 'Get all tickets earned by relaying data packets by your node from every channel.',
  tags: ['Tickets'],
  operationId: 'ticketsGetTickets',
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
