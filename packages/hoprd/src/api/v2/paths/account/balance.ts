import type Hopr from '@hoprnet/hopr-core'
import { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../'

/**
 * @returns Current HOPR and native balance.
 */
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
      const { native, hopr } = await getBalances(node)
      return res.status(200).send({
        native: native.toBN().toString(),
        hopr: hopr.toBN().toString()
      })
    } catch (err) {
      return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
    }
  }
]

GET.apiDoc = {
  description:
    "Check this node's current HOPR and native balances. Hopr tokens from this balance is used to fund payment channels between this node and other nodes on the network. ETH from Native token balance is used to pay for the gas fees on ethereum network.",
  tags: ['Account'],
  operationId: 'getBalance',
  responses: {
    '200': {
      description: 'Balances fetched successfuly.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              balances: { $ref: '#/components/schemas/Balances' }
            }
          }
        }
      }
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
