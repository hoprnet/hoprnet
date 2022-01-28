import { Operation } from 'express-openapi'
import { isError } from '../../../commands/v2'
import { withdraw } from '../../../commands/v2/logic/withdraw'

export const POST: Operation = [
  async (req, res, _next) => {
    try {
      const { commands } = req.context
      const { currency, amount, recipient } = req.body

      const err = await withdraw({
        rawCurrency: currency,
        rawRecipient: recipient,
        rawAmount: amount,
        node: commands.node
      })

      if (isError(err)) {
        return res.status(400).send({ status: err.message })
      } else {
        return res.status(200).send({ status: 'success' })
      }
    } catch (error) {
      return res.status(500).send({ status: 'failure' })
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
            $ref: '#/components/schemas/StatusResponse'
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
