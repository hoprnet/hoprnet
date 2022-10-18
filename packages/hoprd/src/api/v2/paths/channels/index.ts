import type { Operation } from 'express-openapi'
import type { default as Hopr } from '@hoprnet/hopr-core'
import {
  type ChannelEntry,
  ChannelStatus,
  defer,
  generateChannelId,
  PublicKey,
  channelStatusToString,
  type DeferType
} from '@hoprnet/hopr-utils'
import { PeerId } from '@libp2p/interface-peer-id'
import { peerIdFromString } from '@libp2p/peer-id'
import BN from 'bn.js'
import { STATUS_CODES } from '../../utils.js'

export interface ChannelInfo {
  type: 'outgoing' | 'incoming'
  channelId: string
  peerId: string
  status: string
  balance: string
}

export const formatOutgoingChannel = (channel: ChannelEntry): ChannelInfo => {
  return {
    type: 'outgoing',
    channelId: channel.getId().toHex(),
    peerId: channel.source.toPeerId().toString(),
    status: channelStatusToString(channel.status),
    balance: channel.balance.toBN().toString()
  }
}

export const formatIncomingChannel = (channel: ChannelEntry): ChannelInfo => {
  return {
    type: 'incoming',
    channelId: channel.getId().toHex(),
    peerId: channel.destination.toPeerId().toString(),
    status: channelStatusToString(channel.status),
    balance: channel.balance.toBN().toString()
  }
}

const openingRequests = new Map<string, DeferType<void>>()

/**
 * @returns List of incoming and outgoing channels associated with the node.
 */
export const getChannels = async (node: Hopr, includingClosed: boolean) => {
  const selfPubKey = PublicKey.fromPeerId(node.getId())
  const selfAddress = selfPubKey.toAddress()

  const channelsFrom: ChannelInfo[] = (await node.getChannelsFrom(selfAddress))
    .filter((channel) => includingClosed || channel.status !== ChannelStatus.Closed)
    .map(formatOutgoingChannel)

  const channelsTo: ChannelInfo[] = (await node.getChannelsTo(selfAddress))
    .filter((channel) => includingClosed || channel.status !== ChannelStatus.Closed)
    .map(formatIncomingChannel)

  return { incoming: channelsTo, outgoing: channelsFrom }
}

const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { includingClosed } = req.query

    try {
      const channels = await getChannels(node, includingClosed === 'true')
      return res.status(200).send(channels)
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

GET.apiDoc = {
  description:
    'Lists all active channels between this node and other nodes on the Hopr network. By default response will contain all incomming and outgoing channels that are either open, waiting to be opened, or waiting to be closed. If you also want to receive past channels that were closed, you can pass `includingClosed` in the request url query.',
  tags: ['Channels'],
  operationId: 'channelsGetChannels',
  parameters: [
    {
      in: 'query',
      name: 'includingClosed',
      description:
        'When includingClosed is passed the response will include closed channels which are ommited by default.',
      schema: {
        type: 'string',
        example: 'false'
      }
    }
  ],
  responses: {
    '200': {
      description: 'Channels fetched succesfully.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              incoming: {
                type: 'array',
                items: { $ref: '#/components/schemas/Channel' },
                description:
                  'Incomming channels are the ones that were opened by a different node and this node acts as relay.'
              },
              outgoing: {
                type: 'array',
                items: { $ref: '#/components/schemas/Channel' },
                description:
                  'Outgoing channels are the ones that were opened by this node and is using other node as relay.'
              }
            }
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

async function validateOpenChannelParameters(
  node: Hopr,
  counterpartyStr: string,
  amountStr: string
): Promise<
  | {
      valid: false
      reason: keyof typeof STATUS_CODES
    }
  | {
      valid: true
      counterparty: PeerId
      amount: BN
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

  let amount: BN
  try {
    amount = new BN(amountStr)
  } catch {
    return {
      valid: false,
      reason: STATUS_CODES.INVALID_AMOUNT
    }
  }

  const balance = await node.getBalance()
  if (amount.lten(0) || balance.toBN().lt(amount)) {
    return {
      valid: false,
      reason: STATUS_CODES.NOT_ENOUGH_BALANCE
    }
  }

  return {
    valid: true,
    amount,
    counterparty
  }
}

/**
 * Opens channel between two parties.
 * @returns The PeerId associated with the alias.
 */
export async function openChannel(
  node: Hopr,
  counterpartyStr: string,
  amountStr: string
): Promise<
  | {
      success: false
      reason: keyof typeof STATUS_CODES
    }
  | {
      success: true
      channelId: string
      receipt: string
    }
> {
  const validationResult = await validateOpenChannelParameters(node, counterpartyStr, amountStr)

  if (validationResult.valid == false) {
    return { success: false, reason: validationResult.reason }
  }

  const channelId = generateChannelId(
    node.getEthereumAddress(),
    PublicKey.fromPeerId(validationResult.counterparty).toAddress()
  )

  let openingRequest = openingRequests.get(channelId.toHex())

  if (openingRequest == null) {
    openingRequest = defer<void>()
    openingRequests.set(channelId.toHex(), openingRequest)
  } else {
    await openingRequest.promise
  }

  try {
    const { channelId, receipt } = await node.openChannel(validationResult.counterparty, validationResult.amount)
    return { success: true, channelId: channelId.toHex(), receipt }
  } catch (err) {
    const errString = err instanceof Error ? err.message : err?.toString?.() ?? STATUS_CODES.UNKNOWN_FAILURE

    if (errString.includes('Channel is already opened')) {
      return { success: false, reason: STATUS_CODES.CHANNEL_ALREADY_OPEN }
    } else {
      return { success: false, reason: STATUS_CODES.UNKNOWN_FAILURE }
    }
  } finally {
    openingRequests.delete(channelId.toHex())
    openingRequest.resolve()
  }

  // @TODO: handle errors from open channel, inconsistent return value
}

const POST: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { peerId, amount } = req.body

    const openingResult = await openChannel(node, peerId, amount)

    if (openingResult.success == true) {
      res.status(201).send({ channelId: openingResult.channelId, receipt: openingResult.receipt })
    } else {
      switch (openingResult.reason) {
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

// TODO: return tx hash
POST.apiDoc = {
  description:
    'Opens a payment channel between this node and the counter party provided. This channel can be used to send messages between two nodes using other nodes on the network to relay the messages. Each message will deduce its cost from the funded amount to pay other nodes for relaying your messages. Opening a channel can take a little bit of time, because it requires some block confirmations on the blockchain.',
  tags: ['Channels'],
  operationId: 'channelsOpenChannel',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          required: ['peerId', 'amount'],
          properties: {
            peerId: {
              format: 'peerId',
              type: 'string',
              description: 'PeerId that we want to transact with using this channel.'
            },
            amount: {
              format: 'amount',
              type: 'string',
              description:
                'Amount of HOPR tokens to fund the channel. It will be used to pay for sending messages through channel'
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
      description: 'Channel succesfully opened.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              channelId: {
                type: 'string',
                example: '0x04e50b7ddce9770f58cebe51f33b472c92d1c40384759f5a0b1025220bf15ec5',
                description: 'Channel ID that can be used in other calls, not to confuse with transaction hash.'
              },
              receipt: {
                type: 'string',
                example: '0x37954ca4a630aa28f045df2e8e604cae22071046042e557355acf00f4ef20d2e',
                description:
                  'Receipt for open channel transaction. Can be used to check status of the smart contract call on blockchain.'
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
            $ref: '#/components/schemas/RequestStatus'
          },
          example: { status: `${STATUS_CODES.INVALID_AMOUNT} | ${STATUS_CODES.INVALID_ADDRESS}` }
        }
      }
    },
    '403': {
      description: 'Failed to open the channel because of insufficient HOPR balance.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: {
                type: 'string',
                example: `${STATUS_CODES.NOT_ENOUGH_BALANCE}`,
                description: `Insufficient balance to open channel. Amount passed in request body exeeds current balance.`
              }
            }
          },
          example: {
            status: STATUS_CODES.NOT_ENOUGH_BALANCE
          }
        }
      }
    },
    '409': {
      description: 'Failed to open the channel because the channel between this nodes already exists.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: {
                type: 'string',
                example: `${STATUS_CODES.CHANNEL_ALREADY_OPEN}`,
                description: `Channel already open. Cannot open more than one channel between two nodes.`
              }
            }
          },
          example: {
            status: STATUS_CODES.CHANNEL_ALREADY_OPEN
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

export default { POST, GET }
