import { Hash, stringToU8a, debug } from '@hoprnet/hopr-utils'
import { STATUS_CODES } from '../../../../utils.js'

import type { Hopr } from '@hoprnet/hopr-core'
import type { Operation } from 'express-openapi'

const log = debug('hoprd:api:v3:channel-ticket-aggregate')

const POST: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context
    const { channelid } = req.params

    try {
      const channelIdHash = Hash.deserialize(stringToU8a(channelid))
      await node.aggregateTickets(channelIdHash)
      return res.status(204).send()
    } catch (err) {
      log(`${err}`)
      const error = err instanceof Error ? err.message : err?.toString?.() ?? 'Unknown error'

      let status = STATUS_CODES.UNKNOWN_FAILURE

      if (error.match(/non-existing channel/)) {
        res.status(404).send()
        return
      } else if (error.match(/not in status OPEN/)) {
        status = STATUS_CODES.CHANNEL_NOT_OPEN
      } else if (error.match(/No tickets found/)) {
        status = STATUS_CODES.TICKETS_NOT_FOUND
      }

      res.status(422).send({ status, error })

      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

POST.apiDoc = {
  description:
    'Takes all acknowledged and winning tickets (if any) from the given channel and aggregates them into a single ticket. Requires cooperation of the ticket issuer.',
  tags: ['Channels'],
  operationId: 'channelsAggregateTickets',
  parameters: [
    {
      in: 'path',
      name: 'channelid',
      required: true,
      schema: {
        format: 'channelid',
        type: 'string'
      }
    }
  ],
  responses: {
    '204': {
      description: 'Tickets successfully aggregated'
    },
    '400': {
      description: 'Invalid channel id.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: {
            status: STATUS_CODES.INVALID_CHANNELID
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
    '404': {
      $ref: '#/components/responses/NotFound'
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
