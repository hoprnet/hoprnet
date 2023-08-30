import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'
import type { Hopr } from '@hoprnet/hopr-core'

const POST: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context

    try {
      console.log(`about to redeem tickets`)
      await node.redeemAllTickets()
      return res.status(204).send()
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

POST.apiDoc = {
  description:
    'Redeems all tickets from all the channels and exchanges them for Hopr tokens. Every ticket have a chance to be winning one, rewarding you with Hopr tokens.',
  tags: ['Tickets'],
  operationId: 'ticketsRedeemTickets',
  responses: {
    '204': {
      description: 'Tickets redeemed succesfully.'
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

export default { POST }
