import type Hopr from '@hoprnet/hopr-core'
import { Operation } from 'express-openapi'
import { STATUS_CODES } from '../'

export const getBalances = async (node: Hopr) => {
  const [nativeBalance, hoprBalance] = await Promise.all([await node.getNativeBalance(), await node.getBalance()])

  return {
    native: nativeBalance,
    hopr: hoprBalance
  }
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context

    try {
      const balances = await getBalances(node)
      return res.status(200).send({
        balances
      })
    } catch (err) {
      return res.status(500).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
    }
  }
]

GET.apiDoc = {
  description: 'Returns your current HOPR and native balance.',
  tags: ['balance'],
  operationId: 'getBalance',
  responses: {
    '200': {
      description: 'Balance fetched successfuly.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              balances: {
                type: 'object',
                properties: {
                  native: { $ref: '#/components/schemas/Balance' },
                  hopr: { $ref: '#/components/schemas/Balance' }
                }
              }
            }
          }
        }
      }
    }
  }
}
