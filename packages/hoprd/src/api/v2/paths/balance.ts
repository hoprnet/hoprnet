import type Hopr from '@hoprnet/hopr-core'
import { Operation } from 'express-openapi'

export const getBalances = async (node: Hopr) => {
  const [nativeBalance, hoprBalance] = await Promise.all([await node.getNativeBalance(), await node.getBalance()])

  return {
    native: nativeBalance,
    hopr: hoprBalance
  }
}

export const parameters = []

export const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context

    try {
      const balances = await getBalances(node)
      return res.status(200).send({
        status: 'success',
        balances
      })
    } catch (err) {
      return res.status(500).send({ error: err.message })
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
