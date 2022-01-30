import type { Operation } from 'express-openapi'
import { Balance, moveDecimalPoint, NativeBalance, Address } from '@hoprnet/hopr-utils'

type WithdrawArgs = {
  amount: string
  weiAmount: string
  currency: 'NATIVE' | 'HOPR'
  recipient: string
}

const validateWithdrawArgs = async ({
  amount,
  currency,
  recipient
}: {
  amount: string
  currency: string
  recipient: string
}): Promise<WithdrawArgs> => {
  const validCurrency = currency.toUpperCase() as 'NATIVE' | 'HOPR'
  if (!['NATIVE', 'HOPR'].includes(validCurrency)) {
    throw Error('incorrectCurrency')
  }

  if (isNaN(Number(amount))) {
    throw Error('incorrectAmount')
  }

  // @TODO: done by express?
  try {
    Address.fromString(recipient)
  } catch (_err) {
    throw Error('incorrectRecipient')
  }

  const weiAmount =
    validCurrency === 'NATIVE'
      ? moveDecimalPoint(amount, NativeBalance.DECIMALS)
      : moveDecimalPoint(amount, Balance.DECIMALS)

  return {
    amount,
    weiAmount,
    currency: validCurrency,
    recipient
  }
}

export const POST: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { currency, amount, recipient } = req.body
    let validated: WithdrawArgs

    try {
      validated = await validateWithdrawArgs({
        amount,
        currency,
        recipient
      })
    } catch (err) {
      return res.status(400).send({ error: err.message })
    }

    try {
      const txHash = await node.withdraw(validated.currency, validated.recipient, validated.amount)
      return res.status(200).send({ status: 'success', receipt: txHash })
    } catch (err) {
      return res.status(500).send({ error: err.message })
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
