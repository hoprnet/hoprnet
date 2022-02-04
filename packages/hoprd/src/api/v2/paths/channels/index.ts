import type { Operation } from 'express-openapi'
import type Hopr from '@hoprnet/hopr-core'
import { ChannelStatus, PublicKey } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import BN from 'bn.js'
import { STATUS_CODES } from '../../'

export interface ChannelInfo {
  type: 'outgoing' | 'incoming'
  channelId: string
  peerId: string
  status: string
  balance: string
}

const channelStatusToString = (status: ChannelStatus): string => {
  if (status === 0) return 'Closed'
  else if (status === 1) return 'WaitingForCommitment'
  else if (status === 2) return 'Open'
  else if (status === 3) return 'PendingToClose'
  return 'Unknown'
}

/**
 * @returns List of incoming and outgoing channels associated with the node.
 */
export const getChannels = async (node: Hopr, includingClosed: boolean) => {
  const selfPubKey = new PublicKey(node.getId().pubKey.marshal())
  const selfAddress = selfPubKey.toAddress()

  const channelsFrom: ChannelInfo[] = (await node.getChannelsFrom(selfAddress))
    .filter((channel) => includingClosed || channel.status !== ChannelStatus.Closed)
    .map((channel) => ({
      type: 'incoming',
      channelId: channel.getId().toHex(),
      peerId: channel.destination.toPeerId().toB58String(),
      status: channelStatusToString(channel.status),
      balance: channel.balance.toBN().toString()
    }))

  const channelsTo: ChannelInfo[] = (await node.getChannelsTo(selfAddress))
    .filter((channel) => includingClosed || channel.status !== ChannelStatus.Closed)
    .map((channel) => ({
      type: 'outgoing',
      channelId: channel.getId().toHex(),
      peerId: channel.source.toPeerId().toB58String(),
      status: channelStatusToString(channel.status),
      balance: channel.balance.toBN().toString()
    }))

  return { incoming: channelsFrom, outgoing: channelsTo }
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { includingClosed } = req.query

    try {
      const channels = await getChannels(node, !!includingClosed)
      return res.status(200).send(channels)
    } catch (err) {
      return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
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

/**
 * Opens channel between two parties.
 * @returns The PeerId associated with the alias.
 */
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
    const { channelId, receipt } = await node.openChannel(counterparty, amount)
    return { channelId: channelId.toHex(), receipt }
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
      const { channelId, receipt } = await openChannel(node, peerId, amount)
      return res.status(201).send({ channelId, receipt })
    } catch (err) {
      const INVALID_ARG = [STATUS_CODES.INVALID_AMOUNT, STATUS_CODES.INVALID_PEERID].find((arg) =>
        err.message.includes(arg)
      )
      if (INVALID_ARG) {
        return res.status(400).send({ status: INVALID_ARG })
      } else if (err.message.includes(STATUS_CODES.CHANNEL_ALREADY_OPEN)) {
        return res.status(403).send({ status: STATUS_CODES.CHANNEL_ALREADY_OPEN })
      } else if (err.message.includes(STATUS_CODES.NOT_ENOUGH_BALANCE)) {
        return res.status(403).send({ status: STATUS_CODES.NOT_ENOUGH_BALANCE })
      } else {
        return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
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
              type: 'string',
              description: 'PeerId that we want to transact with using this channel.'
            },
            amount: {
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
      description:
        'Failed to open the channel either because of insufficient HOPR balance or because channel between this nodes already exists. Check response examples for more info.',
      content: {
        'application/json': {
          schema: {
            oneOf: [
              {
                type: 'object',
                properties: {
                  status: {
                    type: 'string',
                    example: STATUS_CODES.NOT_ENOUGH_BALANCE,
                    description:
                      'Insufficient balance to open channel. Amount passed in request body exeeds current balance.'
                  }
                }
              },
              {
                type: 'object',
                properties: {
                  status: {
                    type: 'string',
                    example: STATUS_CODES.CHANNEL_ALREADY_OPEN,
                    description: 'Channel already open. Cannot open more than one channel between two nodes.'
                  }
                }
              }
            ]
          },
          examples: {
            NOT_ENOUGH_BALANCE: { value: { status: STATUS_CODES.NOT_ENOUGH_BALANCE } },
            CHANNEL_ALREADY_OPEN: {
              value: { status: STATUS_CODES.CHANNEL_ALREADY_OPEN }
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
