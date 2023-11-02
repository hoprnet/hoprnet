import type { Hopr } from '@hoprnet/hopr-core'
import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'
import { debug } from '@hoprnet/hopr-utils'

export const getTicketsStatistics = async (node: Hopr) => {
  const stats = await node.getTicketStatistics()

  return {
    unredeemed: stats.unredeemed,
    unredeemedValue: stats.unredeemedValue.to_string(),
    redeemed: stats.redeemed,
    redeemedValue: stats.redeemedValue.to_string(),
    losingTickets: stats.losing,
    winProportion: stats.winProportion,
    neglected: stats.neglected,
    neglectedValue: stats.neglectedValue.to_string(),
    rejected: stats.rejected,
    rejectedValue: stats.rejectedValue.to_string()
  }
}

const GET: Operation = [
  async (req, res, _next) => {
    const log = debug('hoprd:api:v3:get-statistics')
    const { node }: { node: Hopr } = req.context

    try {
      log(`about to get ticket statistics`)
      const tickets = await getTicketsStatistics(node)
      return res.status(200).send(tickets)
    } catch (err) {
      log(`failed to get ticket statistics: ${err}`)
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

GET.apiDoc = {
  description:
    'Get statistics regarding all your tickets. Node gets a ticket everytime it relays data packet in channel.',
  tags: ['Tickets'],
  operationId: 'ticketsGetStatistics',
  responses: {
    '200': {
      description:
        'Tickets statistics fetched successfully. Check schema for description of every field in the statistics.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              unredeemed: {
                type: 'number',
                description: 'Number of tickets that wait to be redeemed as for Hopr tokens.'
              },
              unredeemedValue: { type: 'string', description: 'Total value of all unredeemed tickets in Hopr tokens.' },
              redeemed: { type: 'number', description: 'Number of tickets already redeemed on this node.' },
              redeemedValue: { type: 'string', description: 'Total value of all redeemed tickets in Hopr tokens.' },
              losingTickets: {
                type: 'number',
                description: `Number of tickets that didn't win any Hopr tokens. To better understand how tickets work read about probabilistic payments (https://docs.hoprnet.org/core/probabilistic-payments)`
              },
              winProportion: {
                type: 'number',
                description:
                  'Proportion of number of winning tickets vs loosing tickets, 1 means 100% of tickets won and 0 means that all tickets were losing ones.'
              },
              neglected: {
                type: 'number',
                description:
                  'Number of tickets that were not redeemed in time before channel was closed. Those cannot be redeemed anymore.'
              },
              neglectedValue: { type: 'string', description: 'Total value of all neglected tickets in Hopr tokens.' },
              rejected: {
                type: 'number',
                description:
                  'Number of tickets that were rejected by the network by not passing validation. In other words tickets that look suspicious and are not eligible for redeeming.'
              },
              rejectedValue: { type: 'string', description: 'Total value of rejected tickets in Hopr tokens' }
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
