import Hopr from '@hoprnet/hopr-core'
import { Operation } from 'express-openapi'

export const redeemTickets = async ({ node }: { node: Hopr }) => {
  try {
    await node.redeemAllTickets()
  } catch (err) {
    throw new Error('failure' + err.message)
  }
}

export const POST: Operation = [
  async (req, res, _next) => {
    const { node } = req.context

    try {
      await redeemTickets({ node })
      return res.status(200).send({ status: 'success' })
    } catch (err) {
      return res.status(500).send({ status: err.message })
    }
  }
]

POST.apiDoc = {
  description: 'Redeems your tickets',
  tags: ['channel'],
  operationId: 'redeemTickets',
  responses: {
    '200': {
      description: 'Tickets redeemed succesfully',
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
