import { Operation } from 'express-openapi'
import { isError } from '../../../commands/v2'
import { getBalances } from '../../../commands/v2/logic/balance'

export const parameters = []

export const GET: Operation = [
  async (req, res, _next) => {
    const { commands } = req.context

    const balances = await getBalances(commands.node)
    if (isError(balances)) {
      return res.status(500).send({ status: 'failure' })
    } else {
      return res.status(200).send({ status: 'success', balances })
    }
  }
]

GET.apiDoc = {
  description: 'Returns your current HOPR and native balance',
  tags: ['balance'],
  operationId: 'getBalance',
  parameters: [],
  responses: {
    '200': {
      description: 'Balance fetched successfuly',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: { type: 'string', example: 'success' },
              balances: {
                $ref: '#/components/schemas/Balance'
              }
            }
          }
        }
      }
    },
    '500': {
      description: 'Failed to get balance.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: 'failure' }
        }
      }
    }
  }
}
