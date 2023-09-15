import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../../../utils.js'
import type { Hopr } from '@hoprnet/hopr-core'
import { Hash, stringToU8a } from '@hoprnet/hopr-utils'

const POST: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context
    const { channelid } = req.params

    try {
      let channel_id = Hash.deserialize(stringToU8a(channelid))

      console.log(`about to aggregate tickets`)
      await node.aggregateTickets(channel_id)
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
    'Takes all acknowledged and winning tickets from the given channel (if any) and aggregates them into a single ticket. Requires cooperation of the ticket issuer.',
  tags: ['Tickets'],
  operationId: 'ticketsAggregateTickets',
  responses: {
    '204': {
      description: 'Tickets successfully aggregated'
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
