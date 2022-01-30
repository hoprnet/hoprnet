import type { Operation } from 'express-openapi'
import type Hopr from '@hoprnet/hopr-core'
import BN from 'bn.js'
import PeerId from 'peer-id'
import { STATUS_CODES } from '../../'

export const openChannel = async (node: Hopr, counterpartyStr: string, amountStr: string) => {
  let counterparty: PeerId
  try {
    counterparty = PeerId.createFromB58String(counterpartyStr)
  } catch (err) {
    throw Error(STATUS_CODES.INVALID_PEERID)
  }

  if (isNaN(Number(amountStr))) {
    throw Error(STATUS_CODES.INVALID_AMOUNT)
  }

  const amount = new BN(amountStr)
  const balance = await node.getBalance()
  if (amount.lten(0) || balance.toBN().lt(amount)) {
    throw Error(STATUS_CODES.NOT_ENOUGH_BALANCE)
  }

  // @TODO: handle errors from open channel
  try {
    const { channelId } = await node.openChannel(counterparty, amount)
    return channelId.toHex()
  } catch (err) {
    if (err.message.includes('Channel is already opened')) {
      throw Error(STATUS_CODES.CHANNEL_ALREADY_OPEN)
    } else {
      throw Error(err.message)
    }
  }
}

export const POST: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { peerId, amount } = req.body

    try {
      const channelId = await openChannel(node, peerId, amount)
      return res.status(200).send({ status: STATUS_CODES.SUCCESS, channelId })
    } catch (err) {
      const INVALID_ARG = [STATUS_CODES.INVALID_AMOUNT, STATUS_CODES.INVALID_ADDRESS].find(err.message)
      if (INVALID_ARG) {
        return res.status(400).send({ STATUS: INVALID_ARG, error: err.message })
      } else if (err.message.includes(STATUS_CODES.CHANNEL_ALREADY_OPEN)) {
        return res.status(304).send({ status: STATUS_CODES.CHANNEL_ALREADY_OPEN })
      } else {
        return res.status(500).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
      }
    }
  }
]

POST.apiDoc = {
  description: 'Opens a payment channel between you and the counter party provided',
  tags: ['channel'],
  operationId: 'openChannel',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          properties: {
            peerId: {
              type: 'string',
              description:
                'peerId that we want to transact with using this channel, in other words a receiver of funds.'
            },
            amount: { type: 'string', description: 'Amount of tokens to fund the channel.' }
          },
          example: {
            peerId: '16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12',
            amount: '0.001'
          }
        }
      }
    }
  },
  responses: {
    '200': {
      description: 'Channel succesfuly opened',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: { type: 'string', example: 'success' },
              channelId: { type: 'string', example: '7b379578588920ca78fbf' }
            }
          }
        }
      }
    },
    '400': {
      description: 'Problem with inputs',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: 'invalidPeerId | invalidAmountToFund' }
        }
      }
    },
    '403': {
      description: 'Channel already open',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: 'channelAlreadyOpen' }
        }
      }
    },
    '500': {
      description: 'Insufficient balance to open channel',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: { type: 'string', example: 'notEnoughFunds' },
              tokensRequired: { type: 'string', example: '10' },
              currentBalance: { type: 'string', example: '9' }
            }
          }
        }
      }
    }
  }
}
