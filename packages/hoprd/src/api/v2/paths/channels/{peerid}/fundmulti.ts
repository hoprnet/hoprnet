import type { Operation } from 'express-openapi'
import type Hopr from '@hoprnet/hopr-core'
import {
  defer,
  generateChannelId,
  PublicKey,
  type DeferType
} from '@hoprnet/hopr-utils'
import { peerIdFromString } from '@libp2p/peer-id'
import { PeerId } from '@libp2p/interface-peer-id'
import BN from 'bn.js'
import { STATUS_CODES } from '../../../utils.js'

const fundingRequests = new Map<string, DeferType<void>>()

async function validateFundChannelMultiParameters(
  node: Hopr,
  counterpartyStr: string,
  outgoingAmountStr: string,
  incomingAmountStr: string
): Promise<
  | {
      valid: false
      reason: keyof typeof STATUS_CODES
    }
  | {
      valid: true
      counterparty: PeerId
      outgoingAmount: BN
      incomingAmount: BN
    }
> {
  let counterparty: PeerId
  try {
    counterparty = peerIdFromString(counterpartyStr)
  } catch (err) {
    return {
      valid: false,
      reason: STATUS_CODES.INVALID_PEERID
    }
  }
  // cannot open channel to self
  if (counterparty.equals(node.getId())) {
    return {
      valid: false,
      reason: STATUS_CODES.INVALID_PEERID
    }
  }

  let incomingAmount: BN
  let outgoingAmount: BN
  try {
    incomingAmount = new BN(incomingAmountStr)
    outgoingAmount = new BN(outgoingAmountStr)
  } catch {
    return {
      valid: false,
      reason: STATUS_CODES.INVALID_AMOUNT
    }
  }

  const totalAmount = incomingAmount.add(outgoingAmount)
  const balance = await node.getBalance()
  if (totalAmount.lten(0) || balance.toBN().lte(totalAmount)) {
    return {
      valid: false,
      reason: STATUS_CODES.NOT_ENOUGH_BALANCE
    }
  }

  return {
    valid: true,
    counterparty,
    outgoingAmount,
    incomingAmount
  }
}


/**
 * Fund channel between two parties (between this node and another one).
 * @returns two channel ids (outgoing and incoming)
 */
export async function fundChannelMulti(
  node: Hopr,
  counterpartyStr: string,
  outgoingAmountStr: string,
  incomingAmountStr: string
): Promise<
  | {
    success: false
    reason: keyof typeof STATUS_CODES
  }
  | {
    success: true
    outgoingChannelId: string
    incomingChannelId: string
    receipt: string
  }
> {
  const validationResult = await validateFundChannelMultiParameters(node, counterpartyStr, outgoingAmountStr, incomingAmountStr)

  if (validationResult.valid == false) {
    return { success: false, reason: validationResult.reason }
  }

  const outgoingChannelId = generateChannelId(
    node.getEthereumAddress(),
    PublicKey.fromPeerId(validationResult.counterparty).toAddress()
  )
  const incomingChannelId = generateChannelId(
    PublicKey.fromPeerId(validationResult.counterparty).toAddress(),
    node.getEthereumAddress()
  )

  let fundingOutgoingChannelRequest = fundingRequests.get(outgoingChannelId.toHex())
  let fundingIncomingChannelRequest = fundingRequests.get(incomingChannelId.toHex())

  if (fundingOutgoingChannelRequest == null && fundingIncomingChannelRequest == null) {
    // when none of the channel has pending request
    fundingOutgoingChannelRequest = defer<void>()
    fundingIncomingChannelRequest = defer<void>()
    fundingRequests.set(outgoingChannelId.toHex(), fundingOutgoingChannelRequest)
    fundingRequests.set(incomingChannelId.toHex(), fundingIncomingChannelRequest)
  } else {
    // wait until both channel requests are resolved
    await Promise.allSettled([fundingOutgoingChannelRequest, fundingIncomingChannelRequest])
  }

  try {
    const receipt = await node.fundChannel(validationResult.counterparty, validationResult.incomingAmount, validationResult.outgoingAmount)
    return { success: true, outgoingChannelId: outgoingChannelId.toHex(), incomingChannelId: incomingChannelId.toHex(), receipt }
  } catch (err) {
    return { success: false, reason: STATUS_CODES.UNKNOWN_FAILURE }
  } finally {
    fundingRequests.delete(outgoingChannelId.toHex())
    fundingRequests.delete(incomingChannelId.toHex())
    fundingOutgoingChannelRequest.resolve()
    fundingIncomingChannelRequest.resolve()
  }
}

const POST: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { peerid } = req.params
    const { outgoingAmount, incomingAmount } = req.body

    const fundingResult = await fundChannelMulti(node, peerid, outgoingAmount, incomingAmount)

    if (fundingResult.success == true) {
      res.status(201).send({ receipt: fundingResult.receipt })
    } else {
      switch (fundingResult.reason) {
        case STATUS_CODES.INVALID_PEERID:
          res.status(400).send({ status: STATUS_CODES.INVALID_PEERID })
          break
        case STATUS_CODES.INVALID_AMOUNT:
          res.status(400).send({ status: STATUS_CODES.INVALID_AMOUNT })
          break
        case STATUS_CODES.NOT_ENOUGH_BALANCE:
          res.status(403).send({ status: STATUS_CODES.NOT_ENOUGH_BALANCE })
          break
        case STATUS_CODES.CHANNEL_ALREADY_OPEN:
          res.status(409).send({ status: STATUS_CODES.CHANNEL_ALREADY_OPEN })
          break
        default:
          res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE })
          break
      }
    }
  }
]

POST.apiDoc = {
  description: 'Fund one or two payment channels between this node and the counter party provided.',
  tags: ['Channels'],
  operationId: 'channelsFundChannels',
  parameters: [
    {
      in: 'path',
      name: 'peerid',
      example: '16Uiu2HAm91QFjPepnwjuZWzK5pb5ZS8z8qxQRfKZJNXjkgGNUAit',
      required: true,
      schema: {
        type: 'string',
        format: 'peerId',
        description: 'PeerId attached to the channel.',
        example: '16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12'
      }
    }
  ],
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          required: ['peerId', 'outgoingAmount', 'incomingAmount'],
          properties: {
            peerId: {
              format: 'peerId',
              type: 'string',
              description: 'PeerId that we want to transact with using this channel.'
            },
            outgoingAmount: {
              format: 'amount',
              type: 'string',
              description:
                'Amount of HOPR tokens to fund the outgoing channel (node -> counterparty). It will be used to pay for sending messages through channel'
            },
            incomingAmount: {
              format: 'amount',
              type: 'string',
              description:
                'Amount of HOPR tokens to fund the incoming channel (counterparty -> node). It will be used to pay for sending messages through channel'
            }
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
    '201': {
      description: 'Channels succesfully funded.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              receipt: {
                type: 'string',
                example: '0x37954ca4a630aa28f045df2e8e604cae22071046042e557355acf00f4ef20d2e',
                description:
                  'Receipt for fund multi channels transaction. Can be used to check status of the smart contract call on blockchain.'
              }
            }
          }
        }
      }
    },
    '400': {
      description: 'Problem with inputs (invalid peerId or invalid amount).',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: {
            status: `${STATUS_CODES.INVALID_PEERID} | ${STATUS_CODES.INVALID_AMOUNT}`
          }
        }
      }
    },
    '403': {
      description: 'Failed to fund the channels because of insufficient HOPR balance.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: {
                type: 'string',
                example: `${STATUS_CODES.NOT_ENOUGH_BALANCE}`,
                description: `Insufficient balance to fund channels. Amount passed in request body exeeds current balance.`
              }
            }
          },
          example: {
            status: STATUS_CODES.NOT_ENOUGH_BALANCE
          }
        }
      }
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
