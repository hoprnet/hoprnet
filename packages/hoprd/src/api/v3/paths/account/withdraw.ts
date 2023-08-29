import type { Operation } from 'express-openapi'
import type { Hopr } from '@hoprnet/hopr-core'
import { Address } from '@hoprnet/hopr-utils'
import { STATUS_CODES } from '../../utils.js'

/**
 * Withdraws specified amount of specified currency from the node.
 * @returns Transaction hash if transaction got successfully submited.
 */
export const withdraw = async (node: Hopr, currency: 'native' | 'hopr', recipient: string, amount: string) => {
  const currencyUpperCase = currency.toUpperCase() as 'NATIVE' | 'HOPR'
  if (!['NATIVE', 'HOPR'].includes(currencyUpperCase)) {
    throw Error(STATUS_CODES.INVALID_CURRENCY)
  }

  if (isNaN(Number(amount)) || !isFinite(Number(amount))) {
    throw Error(STATUS_CODES.INVALID_AMOUNT)
  }

  try {
    Address.from_string(recipient)
  } catch (_err) {
    throw Error(STATUS_CODES.INVALID_ADDRESS)
  }

  const balance = currencyUpperCase === 'NATIVE' ? await node.getNativeBalance() : await node.getBalance()
  if (balance.lt(balance.of_same(amount))) {
    throw Error(STATUS_CODES.NOT_ENOUGH_BALANCE)
  }

  // TODO: withdraw hopr broken, its working but only resolves after transaction have been mined.
  const txHash = await node.withdraw(currencyUpperCase, recipient, amount)
  return txHash
}

const POST: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { currency, amount, ethereumAddress } = req.body

    try {
      const txHash = await withdraw(node, currency, ethereumAddress, amount)
      return res.status(200).send({ receipt: txHash })
    } catch (err) {
      const errString = err instanceof Error ? err.message : err?.toString?.() ?? 'Unknown error'

      return res.status(422).send({
        status: errString.includes(STATUS_CODES.NOT_ENOUGH_BALANCE)
          ? STATUS_CODES.NOT_ENOUGH_BALANCE
          : STATUS_CODES.UNKNOWN_FAILURE,
        error: errString
      })
    }
  }
]

POST.apiDoc = {
  description:
    'Withdraw funds from this node to your ethereum wallet address. You can choose whitch currency you want to withdraw, NATIVE or HOPR.',
  tags: ['Account'],
  operationId: 'accountWithdraw',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          required: ['currency', 'amount', 'ethereumAddress'],
          properties: {
            currency: {
              $ref: '#/components/schemas/Currency'
            },
            amount: {
              type: 'string',
              format: 'amount',
              description: "Amount to withdraw in the currency's smallest unit.",
              example: '1337'
            },
            ethereumAddress: {
              $ref: '#/components/schemas/NativeAddress'
            }
          }
        }
      }
    }
  },
  responses: {
    '200': {
      description:
        'Withdraw successful. Receipt from this response can be used to check details of the transaction on ethereum chain.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              receipt: {
                type: 'string',
                example: '0x37954ca4a630aa28f045df2e8e604cae22071046042e557355acf00f4ef20d2e',
                description: 'Withdraw txn hash that can be used to check details of the transaction on ethereum chain.'
              }
            }
          }
        }
      }
    },
    '400': {
      description: `Incorrect data in request body. Make sure to provide valid currency ('NATIVE' | 'HOPR') or amount.`,
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: {
            status: `${STATUS_CODES.INVALID_CURRENCY} | ${STATUS_CODES.INVALID_AMOUNT}`
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
      description:
        'Withdraw amount exeeds current balance or unknown error. You can check current balance using /account/balance endpoint.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: {
                type: 'string',
                example: `${STATUS_CODES.NOT_ENOUGH_BALANCE} | ${STATUS_CODES.UNKNOWN_FAILURE}`
              },
              error: { type: 'string', example: 'NOT_ENOUGH_BALANCE' }
            }
          },
          example: {
            status: `${STATUS_CODES.NOT_ENOUGH_BALANCE}`
          }
        }
      }
    }
  }
}

export default { POST }
