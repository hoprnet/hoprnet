import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'
import { Hopr } from '@hoprnet/hopr-utils'


const GET: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context

    try {
      const ticket_price = await node.getTicketPrice()
      return res.status(200).send({price: ticket_price})
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

GET.apiDoc = {
  description: 'Get the latest ticket price.',
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
