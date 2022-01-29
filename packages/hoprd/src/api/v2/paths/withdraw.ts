import { Operation } from 'express-openapi'
import { isError } from '../logic'
import { withdraw } from '../logic/withdraw'

export const POST: Operation = [
  async (req, res, _next) => {
    try {
      const { node } = req.context
      const { currency, amount, recipient } = req.body

      const receipt = await withdraw({
        rawCurrency: currency,
        rawRecipient: recipient,
        rawAmount: amount,
        node: node
      })

      if (isError(receipt)) {
        return res.status(400).send({ status: receipt.message })
      } else {
        return res.status(200).send({ status: 'success', receipt })
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
