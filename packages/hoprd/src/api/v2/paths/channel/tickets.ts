import Hopr from '@hoprnet/hopr-core'
import { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../'

export const getTickets = async (node: Hopr) => {
  const stats = await node.getTicketStatistics()

  return {
    pending: stats.pending,
    unredeemed: stats.unredeemed,
    unredeemedValue: stats.unredeemedValue.toFormattedString(),
    redeemed: stats.redeemed,
    redeemedValue: stats.redeemedValue.toFormattedString(),
    losingTickets: stats.losing,
    winProportion: stats.winProportion,
    neglected: stats.neglected,
    rejected: stats.rejected,
    rejectedValue: stats.rejectedValue.toFormattedString()
  }
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context

    try {
      const tickets = await getTickets(node)
      return res.status(200).send(tickets)
    } catch (err) {
      return res.status(500).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
    }
  }
]

GET.apiDoc = {
  description: 'Get statistics regarding your tickets.',
  tags: ['channel'],
  operationId: 'getTickets',
  responses: {
    '200': {
      description: 'Tickets statistics fetched successfully.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              pending: { type: 'number' },
              unredeemed: { type: 'number' },
              unredeemedValue: { type: 'string', example: '0 txHOPR' },
              redeemed: { type: 'number' },
              redeemedValue: { type: 'string', example: '0 txHOPR' },
              losingTickets: { type: 'number' },
              winProportion: { type: 'number' },
              neglected: { type: 'number' },
              rejected: { type: 'number' },
              rejectedValue: { type: 'string', example: '0 txHOPR' }
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
