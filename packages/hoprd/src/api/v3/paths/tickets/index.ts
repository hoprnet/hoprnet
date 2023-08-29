import type { Hopr } from '@hoprnet/hopr-core'
import type { Ticket } from '@hoprnet/hopr-utils'
import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'

export function formatTicket(ticket: Ticket) {
  return {
    channelId: ticket.channel_id.to_hex(),
    amount: ticket.amount.to_formatted_string(),
    index: ticket.index.to_string(),
    indexOffset: ticket.index_offset.to_string(),
    winProb: ticket.win_prob.toString(),
    channelEpoch: ticket.channel_epoch.to_string(),
    signature: ticket.signature?.to_hex()
  }
}

export async function getAllTickets(node: Hopr) {
  const tickets = await node.getAllTickets()
  return tickets.map(formatTicket)
}

const GET: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context

    try {
      const tickets = await getAllTickets(node)
      return res.status(200).send(tickets)
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

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
