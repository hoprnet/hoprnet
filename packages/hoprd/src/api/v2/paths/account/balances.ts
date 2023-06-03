import type Hopr from '@hoprnet/hopr-core'
import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'

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

const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context

    try {
      const { native, hopr } = await getBalances(node)
      return res.status(200).send({
        native: native.to_string(),
        hopr: hopr.to_string()
      })
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

GET.apiDoc = {
  description:
    "Get node's HOPR and native balances. HOPR tokens from this balance is used to fund payment channels between this node and other nodes on the network. NATIVE balance is used to pay for the gas fees for the blockchain.",
  tags: ['Account'],
  operationId: 'accountGetBalances',
  responses: {
    '200': {
      description: 'Balances fetched successfuly.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              native: {
                $ref: '#/components/schemas/NativeBalance'
              },
              hopr: {
                $ref: '#/components/schemas/HoprBalance'
              }
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
