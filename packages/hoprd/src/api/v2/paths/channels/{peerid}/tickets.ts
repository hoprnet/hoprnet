import { Operation } from 'express-openapi'
// import PeerId from 'peer-id'
import { STATUS_CODES } from '../../../'

export const GET: Operation = [
  async (req, res, _next) => {
    // const { node } = req.context
    const { peerid } = req.params

    try {
      // TODO: implement logic for getting ticktes for specific channel
      const tickets = await new Promise((resolve) => resolve(peerid))
      return res.status(200).send(tickets)
    } catch (err) {
      return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
    }
  }
]

GET.apiDoc = {
  description: 'Get tickets earned by relaying data packets by your node for the particular channel.',
  tags: ['Channels'],
  operationId: 'getTickets',
  responses: {
    '200': {
      description: 'Tickets fetched successfully.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              someTicketsHere: { type: 'string' }
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
