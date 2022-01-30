import { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../'

export const POST: Operation = [
  async (req, res, _next) => {
    const { node } = req.context

    try {
      await node.redeemAllTickets()
      return res.status(200).send({ status: STATUS_CODES.SUCCESS })
    } catch (err) {
      return res.status(500).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
    }
  }
]

POST.apiDoc = {
  description: 'Redeems your tickets.',
  tags: ['channel'],
  operationId: 'channelRedeemTickets',
  responses: {
    '200': {
      description: 'Tickets redeemed succesfully.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          }
        }
      }
    }
  }
}
