import type { Operation } from 'express-openapi'
import type Hopr from '@hoprnet/hopr-core'
import {
  type ChannelEntry,
  ChannelStatus,
  defer,
  generate_channel_id,
  PublicKey,
  channel_status_to_string,
  type DeferType,
  Address
} from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { STATUS_CODES } from '../../utils.js'

export interface ChannelInfo {
  type: 'outgoing' | 'incoming'
  channelId: string
  address: string
  status: string
  balance: string
}

export interface ChannelTopologyInfo {
  channelId: string
  sourceAddress: string
  destinationAddress: string
  balance: string
  status: string
  ticketIndex: string
  channelEpoch: string
  closureTime: string
}

/**
 * Format channel entries
 * @param channel channelEntry entity saved in the database
 * @returns stringified fields from ChannelEntry and both peer id and address for source/destination
 */
export const formatChannelTopologyInfo = (channel: ChannelEntry): ChannelTopologyInfo => {
  return {
    channelId: channel.get_id().to_hex(),
    sourceAddress: channel.source.to_hex(),
    destinationAddress: channel.destination.to_hex(),
    balance: channel.balance.to_string(),
    status: channel_status_to_string(channel.status),
    ticketIndex: channel.ticket_index.to_string(),
    channelEpoch: channel.channel_epoch.to_string(),
    closureTime: channel.closure_time.to_string()
  }
}

export const formatOutgoingChannel = (channel: ChannelEntry): ChannelInfo => {
  return {
    type: 'outgoing',
    channelId: channel.get_id().to_hex(),
    address: channel.destination.to_string(),
    status: channel_status_to_string(channel.status),
    balance: channel.balance.to_string()
  }
}

export const formatIncomingChannel = (channel: ChannelEntry): ChannelInfo => {
  return {
    type: 'incoming',
    channelId: channel.get_id().to_hex(),
    address: channel.source.to_string(),
    status: channel_status_to_string(channel.status),
    balance: channel.balance.to_string()
  }
}

const openingRequests = new Map<string, DeferType<void>>()

/**
 * @returns List of incoming and outgoing channels associated with the node.
 */
export const getChannels = async (node: Hopr, includingClosed: boolean) => {
  const selfPubKey = PublicKey.from_peerid_str(node.getId().toString())
  const selfAddress = selfPubKey.to_address()

  const channelsFrom: ChannelInfo[] = (await node.getChannelsFrom(selfAddress))
    .filter((channel) => includingClosed || channel.status !== ChannelStatus.Closed)
    .map(formatOutgoingChannel)

  const channelsTo: ChannelInfo[] = (await node.getChannelsTo(selfAddress))
    .filter((channel) => includingClosed || channel.status !== ChannelStatus.Closed)
    .map(formatIncomingChannel)

  return { incoming: channelsTo, outgoing: channelsFrom, all: [] }
}

/**
 * @returns List of all the channels
 */
export const getAllChannels = async (node: Hopr) => {
  const channels = await node.getAllChannels()
  const channelTopology: ChannelTopologyInfo[] = channels.map(formatChannelTopologyInfo)

  return { incoming: [], outgoing: [], all: channelTopology }
}

const GET: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context
    const { includingClosed, fullTopology } = req.query

    try {
      let channels
      if (fullTopology === 'true') {
        channels = await getAllChannels(node)
      } else {
        channels = await getChannels(node, includingClosed === 'true')
      }

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
    },
    {
      in: 'query',
      name: 'fullTopology',
      description: 'Get the full payment channel graph indexed by the node.',
      schema: {
        type: 'string',
        example: 'false'
      }
    }
  ],
  responses: {
    '200': {
      description: 'Channels fetched successfully.',
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
              },
              all: {
                type: 'array',
                items: { $ref: '#/components/schemas/ChannelTopology' },
                description: 'All the channels indexed by the node in the current network.'
              }
            }
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
      counterparty: Address
      amount: BN
    }
> {
  let counterparty: Address
  try {
    counterparty = PublicKey.from_peerid_str(counterpartyStr).to_address()
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
  if (amount.lten(0) || balance.lt(balance.of_same(amount.toString()))) {
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

  const channelId = generate_channel_id(node.getEthereumAddress(), validationResult.counterparty)

  let openingRequest = openingRequests.get(channelId.to_hex())

  if (openingRequest == null) {
    openingRequest = defer<void>()
    openingRequests.set(channelId.to_hex(), openingRequest)
  } else {
    await openingRequest.promise
  }

  try {
    const { channelId, receipt } = await node.openChannel(validationResult.counterparty, validationResult.amount)
    return { success: true, channelId: channelId.to_hex(), receipt }
  } catch (err) {
    const errString = err instanceof Error ? err.message : err?.toString?.() ?? STATUS_CODES.UNKNOWN_FAILURE

    if (errString.includes('Channel is already opened')) {
      return { success: false, reason: STATUS_CODES.CHANNEL_ALREADY_OPEN }
    } else {
      return { success: false, reason: STATUS_CODES.UNKNOWN_FAILURE }
    }
  } finally {
    openingRequests.delete(channelId.to_hex())
    openingRequest.resolve()
  }

  // @TODO: handle errors from open channel, inconsistent return value
}

const POST: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context
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
    '401': {
      $ref: '#/components/responses/Unauthorized'
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
