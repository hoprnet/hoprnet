import type { Operation } from 'express-openapi'
import type Hopr from '@hoprnet/hopr-core'
import { Address } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { STATUS_CODES } from '../../'

/**
 * Withdraws specified amount of specified currency from the node.
 * @returns Transaction hash if transaction got successfully submited.
 */
export const withdraw = async (node: Hopr, currency: 'native' | 'hopr', recipient: string, amount: string) => {
  const currencyUpperCase = currency.toUpperCase() as 'NATIVE' | 'HOPR'
  if (!['NATIVE', 'HOPR'].includes(currencyUpperCase)) {
    throw Error(STATUS_CODES.INVALID_CURRENCY)
  }

  if (isNaN(Number(amount))) {
    throw Error(STATUS_CODES.INVALID_AMOUNT)
  }

  try {
    Address.fromString(recipient)
  } catch (_err) {
    throw Error(STATUS_CODES.INVALID_ADDRESS)
  }

  const balance = currencyUpperCase === 'NATIVE' ? await node.getNativeBalance() : await node.getBalance()
  if (balance.toBN().lt(new BN(amount))) {
    throw Error(STATUS_CODES.NOT_ENOUGH_BALANCE)
  }

  // TODO: withdraw hopr broken, its working but only resolves after transaction have been mined.
  const txHash = await node.withdraw(currencyUpperCase, recipient, amount)
  return txHash
}

export const POST: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { currency, amount, recipient } = req.body

    try {
      const txHash = await withdraw(node, currency, recipient, amount)
      return res.status(200).send({ receipt: txHash })
    } catch (err) {
      const INVALID_ARG = [
        STATUS_CODES.INVALID_CURRENCY,
        STATUS_CODES.INVALID_AMOUNT,
        STATUS_CODES.INVALID_ADDRESS
      ].find((arg) => err.message.includes(arg))
      if (INVALID_ARG) {
        return res.status(400).send({ STATUS: INVALID_ARG, error: err.message })
      } else {
        return res.status(422).send({
          STATUS: err.message.includes(STATUS_CODES.NOT_ENOUGH_BALANCE)
            ? STATUS_CODES.NOT_ENOUGH_BALANCE
            : STATUS_CODES.UNKNOWN_FAILURE,
          error: err.message
        })
      }
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
          required: ['currency', 'amount', 'recipient'],
          properties: {
            currency: {
              $ref: '#/components/schemas/Currency'
            },
            amount: {
              type: 'string',
              description: "Amount to withdraw in the currency's smallest unit.",
              example: '1337'
            },
            recipient: {
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
        'Withdraw successful. Receipt from this response can be used to check details of the transaction on ethereum network.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              receipt: {
                type: 'string',
                example: '0x37954ca4a630aa28f045df2e8e604cae22071046042e557355acf00f4ef20d2e',
                description:
                  'Withdraw txn hash that can be used to check details of the transaction on ethereum network.'
              }
            }
          }
        }
      }
    },
    '400': {
      description: `Incorrect data in request body. Make sure to provide valid currency ('NATIVE' | 'HOPR'), amount, and ethereum address.`,
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: {
            status: `${STATUS_CODES.INVALID_CURRENCY} | ${STATUS_CODES.INVALID_AMOUNT} | ${STATUS_CODES.INVALID_ADDRESS}`
          }
        }
      }
    },
    '422': {
      description:
        'Withdraw amount exeeds current balance. You can check current balance using /account/balance endpoint.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: {
            status: `${STATUS_CODES.NOT_ENOUGH_BALANCE}`
          }
        }
      }
    }
  }
}
