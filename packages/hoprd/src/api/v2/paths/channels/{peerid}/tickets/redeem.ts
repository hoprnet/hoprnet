import { Operation } from 'express-openapi'
import PeerId from 'peer-id'
import { STATUS_CODES } from '../../../../'

export const POST: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { peerid } = req.params

    try {
      await node.redeemTicketsInChannel(PeerId.createFromB58String(peerid))
      return res.status(200).send()
    } catch (err) {
      return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
    }
  }
]

POST.apiDoc = {
  description: 'Redeems your tickets for this channel.',
  tags: ['Channels'],
  operationId: 'channelRedeemTickets',
  responses: {
    '200': {
      description: 'Tickets redeemed succesfully.'
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
