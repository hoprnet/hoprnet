import { Operation } from 'express-openapi'
import PeerId from 'peer-id'
import { STATUS_CODES } from '../../../../'

export const POST: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { peerid } = req.params

    try {
      await node.redeemTicketsInChannel(PeerId.createFromB58String(peerid))
      return res.status(204).send()
    } catch (err) {
      return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
    }
  }
]

POST.apiDoc = {
  description:
    'Redeems your tickets for this channel. Redeeming will change your tickets into Hopr tokens if they are winning ones. You can check how much tickets given channel has by calling /channels/{peerid}/tickets endpoint. Do this before channel is closed as neglected tickets are no longer valid for redeeming.',
  tags: ['Channels'],
  operationId: 'channelsRedeemTickets',
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
