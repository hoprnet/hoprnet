import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'
import { debug, Hopr } from '@hoprnet/hopr-utils'


const GET: Operation = [
  async (req, res, _next) => {
    const log = debug('hoprd:api:v3:get-ticket-price')
    const { node }: { node: Hopr } = req.context

    try {
      const ticket_price = await node.getTicketPrice()
      log(`retrieved ticket price ${ticket_price}`)

      if (ticket_price === null) {
        return res
          .status(206)
          .send({ status: STATUS_CODES.TICKET_PRICE_NOT_FOUND, error: 'Could not retrieve ticket price' })
      }

      return res.status(200).send({price: ticket_price})
      
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

GET.apiDoc = {
  description: 'Get the latest ticket price in wei',
  tags: ['Tickets'],
  operationId: 'ticketsGetTicketPrice',
  responses: {
    '200': {
      description: 'Price fetched successfully.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              price: {
                type: 'number',
                description: 'Latest ticket price update.'
              },
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
