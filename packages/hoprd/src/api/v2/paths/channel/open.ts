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

  // @TODO: handle errors from open channel, inconsistent return value
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
      return res.status(200).send({ channelId })
    } catch (err) {
      const INVALID_ARG = [STATUS_CODES.INVALID_AMOUNT, STATUS_CODES.INVALID_PEERID].find((arg) =>
        err.message.includes(arg)
      )
      if (INVALID_ARG) {
        return res.status(400).send({ status: INVALID_ARG })
      } else if (err.message.includes(STATUS_CODES.CHANNEL_ALREADY_OPEN)) {
        return res.status(304).send({ status: STATUS_CODES.CHANNEL_ALREADY_OPEN })
      } else if (err.message.includes(STATUS_CODES.NOT_ENOUGH_BALANCE)) {
        return res.status(403).send({ status: STATUS_CODES.NOT_ENOUGH_BALANCE })
      } else {
        return res.status(500).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
      }
    }
  }
]

POST.apiDoc = {
  description: 'Opens a payment channel between you and the counter party provided.',
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
              description: 'PeerId that we want to transact with using this channel.'
            },
            amount: { type: 'string', description: 'Amount of tokens to fund the channel.' }
          },
          example: {
            peerId: '16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12',
            amount: '1000000'
          }
        }
      }
    }
  },
  responses: {
    '200': {
      description: 'Channel succesfully opened.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              channelId: {
                type: 'string',
                example: '0x04e50b7ddce9770f58cebe51f33b472c92d1c40384759f5a0b1025220bf15ec5'
              }
            }
          }
        }
      }
    },
    '400': {
      description: 'Problem with inputs.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: `${STATUS_CODES.INVALID_AMOUNT} | ${STATUS_CODES.INVALID_ADDRESS}` }
        }
      }
    },
    '304': {
      description: 'Channel already open.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: STATUS_CODES.CHANNEL_ALREADY_OPEN }
        }
      }
    },
    '403': {
      description: 'Insufficient balance to open channel.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: { status: STATUS_CODES.NOT_ENOUGH_BALANCE }
        }
      }
    },
    '500': {
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
