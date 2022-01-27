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
  parameters: [
    {
      name: 'body',
      in: 'body',
      required: true,
      schema: {
        type: 'object',
        properties: {
          currency: { type: 'string', description: 'ETH | HOPR currency to withdraw.' },
          amount: { type: 'string', description: 'Amount to withdraw.' },
          recipient: { type: 'string', description: 'Blockchain address to withdraw specified currency to.' }
        },
        example: {
          currency: 'hopr',
          amount: '1',
          recipient: '0x2C505741584f8591e261e59160D0AED5F74Dc29b'
        }
      }
    }
  ],
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
          example: { status: 'insufficentBalance | failure' }
        }
      }
    }
  }
}
