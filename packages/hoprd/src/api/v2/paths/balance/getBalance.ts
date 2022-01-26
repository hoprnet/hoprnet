import { Operation } from 'express-openapi'
import { isError } from '../../../../commands/v2'
import { getBalances } from '../../../../commands/v2/logic/balance'

export const parameters = []

export const GET: Operation = [
  async (req, res, _next) => {
    const { commands } = req.context

    const balances = getBalances(commands.node)
    if (isError(balances)) {
      return res.status(500).send({ status: 'failure' })
    } else {
      return res.status(200).send({ status: 'success', balances })
    }
  }
]

GET.apiDoc = {
  description: 'Get the native and hopr addresses of the account associated with the node',
  tags: ['account'],
  operationId: 'accountGetAddress',
  parameters: [],
  responses: {
    '200': {
      description: 'Returns the native and hopr addresses of the account associated with the node',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/Address'
          }
        }
      }
    }
  }
}
