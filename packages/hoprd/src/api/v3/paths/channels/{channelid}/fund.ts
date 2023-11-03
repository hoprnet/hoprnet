import BN from 'bn.js'

import { Balance, BalanceType, debug, Hash, stringToU8a, Hopr } from '@hoprnet/hopr-utils'

import { STATUS_CODES } from '../../../utils.js'

import type { Operation } from 'express-openapi'

const log = debug('hoprd:api:v3:channel-fund')

const POST: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context
    const { channelid } = req.params
    const { amount } = req.body

    if (!amount) {
      res.status(400).send({ status: STATUS_CODES.INVALID_AMOUNT, error: 'amount missing' })
      return
    }

    try {
      const channelIdHash = Hash.deserialize(stringToU8a(channelid))
      let value = new Balance(new BN(amount).toString(10), BalanceType.HOPR)
      const receipt: Hash = await node.fundChannel(channelIdHash, value)
      res.status(200).send({ receipt: receipt.to_hex() })
    } catch (err) {
      log(`${err}`)
      const error = err instanceof Error ? err.message : err?.toString?.() ?? 'Unknown error'

      let status = STATUS_CODES.UNKNOWN_FAILURE

      if (error.match(/non-existing channel/)) {
        res.status(404).send()
        return
      } else if (error.match(/not in status OPEN/)) {
        status = STATUS_CODES.CHANNEL_NOT_OPEN
      } else if (error.match(/must be more than 0/)) {
        status = STATUS_CODES.AMOUNT_TOO_SMALL
      } else if (error.match(/Not enough balance/)) {
        status = STATUS_CODES.NOT_ENOUGH_BALANCE
      } else if (error.match(/Not enough allowance/)) {
        status = STATUS_CODES.NOT_ENOUGH_ALLOWANCE
      }

      res.status(422).send({ status, error })
    }
  }
]

POST.apiDoc = {
  description: `Funds an existing channel with the given amount. The channel must be in state OPEN`,
  tags: ['Channels'],
  operationId: 'channelsFundChannel',
  parameters: [
    {
      in: 'path',
      name: 'channelid',
      required: true,
      schema: {
        format: 'channelid',
        type: 'string'
      }
    }
  ],
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          required: ['amount'],
          properties: {
            amount: {
              format: 'amount',
              type: 'string',
              description:
                'Amount of weiHOPR tokens to fund the channel. It will be used to pay for sending messages through channel'
            }
          },
          example: {
            amount: '1000000'
          }
        }
      }
    }
  },
  responses: {
    '200': {
      description: 'Channel funded successfully.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            required: ['receipt'],
            properties: {
              receipt: {
                type: 'string',
                description: 'Receipt of the funding transaction',
                example: '0x37954ca4a630aa28f045df2e8e604cae22071046042e557355acf00f4ef20d2e'
              }
            }
          }
        }
      }
    },
    '400': {
      description: 'Invalid channel id.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: {
            status: STATUS_CODES.INVALID_CHANNELID
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
    '404': {
      $ref: '#/components/responses/NotFound'
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

export default { POST }
