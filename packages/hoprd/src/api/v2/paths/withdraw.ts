import type { Operation } from 'express-openapi'
import type Hopr from '@hoprnet/hopr-core'
import { Address } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { STATUS_CODES } from '../'

export const withdraw = async (node: Hopr, rawCurrency: 'NATIVE' | 'HOPR', recipient: string, amount: string) => {
  const currency = rawCurrency.toUpperCase() as 'NATIVE' | 'HOPR'
  if (!['NATIVE', 'HOPR'].includes(currency)) {
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

  const balance = currency === 'NATIVE' ? await node.getNativeBalance() : await node.getBalance()
  if (balance.toBN().lt(new BN(amount))) {
    throw Error(STATUS_CODES.NOT_ENOUGH_BALANCE)
  }

  const txHash = await node.withdraw(currency, recipient, amount)
  return txHash
}

export const POST: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { currency, amount, recipient } = req.body

    try {
      const txHash = await withdraw(node, currency, recipient, amount)
      return res.status(200).send({ status: STATUS_CODES.SUCCESS, receipt: txHash })
    } catch (err) {
      const INVALID_ARG = [
        STATUS_CODES.INVALID_CURRENCY,
        STATUS_CODES.INVALID_AMOUNT,
        STATUS_CODES.INVALID_ADDRESS
      ].find(err.message)
      if (INVALID_ARG) {
        return res.status(400).send({ STATUS: INVALID_ARG, error: err.message })
      } else {
        return res.status(500).send({
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
  description: 'Withdraw native or hopr to a specified recipient',
  tags: ['balance'],
  operationId: 'withdraw',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          $ref: '#/components/schemas/WithdrawRequest'
        }
      }
    }
  },
  responses: {
    '200': {
      description: 'Withdraw successful',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: { type: 'string', example: 'success' },
              receipt: { type: 'string', example: '0xc0d8dcb4c83543adfd77b44390d2b61bc28ebe6585a6b1a30550987af9798448' }
            }
          }
        }
      }
    },
    '400': {
      description: 'Incorrect data in request body',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: 'incorrectCurrency | incorrectAmount' }
        }
      }
    },
    '500': {
      description: 'Withdraw failed',
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
