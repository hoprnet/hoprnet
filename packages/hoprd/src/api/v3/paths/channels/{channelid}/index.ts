import { Hash, channel_status_to_string, stringToU8a, ChannelStatus, defer, type DeferType } from '@hoprnet/hopr-utils'

import { STATUS_CODES } from '../../../utils.js'
import { ChannelTopologyInfo, formatChannelTopologyInfo } from '../index.js'

import type { Hopr } from '@hoprnet/hopr-core'
import type { Operation } from 'express-openapi'

import { log } from 'debug'

const closingRequests = new Map<string, DeferType<void>>()

/**
 * Closes a channel with channel id.
 * @returns Channel status and receipt.
 */
export async function closeChannel(
  node: Hopr,
  channelIdStr: string
): Promise<
  | {
      success: false
      reason: keyof typeof STATUS_CODES
    }
  | {
      success: true
      channelStatus: ChannelStatus
      receipt: string
    }
> {
  const channelIdHash = Hash.deserialize(stringToU8a(channelIdStr))
  const channel = await node.db.get_channel(channelIdHash)
  const channelId = channel.get_id()

  let closingRequest = closingRequests.get(channelId.to_hex())
  if (closingRequest == null) {
    closingRequest = defer<void>()
    closingRequests.set(channelId.to_hex(), closingRequest)
  } else {
    await closingRequest.promise
  }

  // incoming if destination is me, outgoing if source is me
  let direction
  let counterpartyAddress
  if (channel.source == node.getEthereumAddress()) {
    direction = 'outgoing'
    counterpartyAddress = channel.destination
  } else if (channel.destination == node.getEthereumAddress()) {
    direction = 'incoming'
    counterpartyAddress = channel.source
  } else {
    return { success: false, reason: STATUS_CODES.UNSUPPORTED_FEATURE }
  }

  try {
    const { status: channelStatus, receipt } = await node.closeChannel(counterpartyAddress, direction)
    return { success: true, channelStatus, receipt }
  } catch (err) {
    log(`${err}`)
    const errString = err instanceof Error ? err.message : err?.toString?.() ?? 'Unknown error'

    if (errString.match(/Channel is already closed/)) {
      // @TODO insert receipt
      return { success: true, receipt: /* @fixme */ '0x', channelStatus: ChannelStatus.Closed }
    } else if (errString.includes('Incoming channel')) {
      return { success: false, reason: STATUS_CODES.UNSUPPORTED_FEATURE }
    } else {
      return { success: false, reason: STATUS_CODES.UNKNOWN_FAILURE }
    }
  } finally {
    closingRequests.delete(channelId.to_hex())
    closingRequest.resolve()
  }
}

const DELETE: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context
    const { channelid } = req.params

    const closingResult = await closeChannel(node, channelid)

    if (closingResult.success == true) {
      res
        .status(200)
        .send({ receipt: closingResult.receipt, channelStatus: channel_status_to_string(closingResult.channelStatus) })
    } else {
      res.status(422).send({ status: closingResult.reason })
    }
  }
]

DELETE.apiDoc = {
  description: `Close a opened channel between this node and other node. Once you've initiated channel closure, you have to wait for a specified closure time, it will show you a closure initiation message with cool-off time you need to wait.
  Then you will need to send the same command again to finalize closure. This is a cool down period to give the other party in the channel sufficient time to redeem their tickets.`,
  tags: ['Channels'],
  operationId: 'channelsCloseChannel',
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
  responses: {
    '200': {
      description: 'Channel closed succesfully.',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              receipt: {
                type: 'string',
                description: 'Receipt of the closing transaction',
                example: '0x37954ca4a630aa28f045df2e8e604cae22071046042e557355acf00f4ef20d2e'
              },
              channelStatus: { type: 'string', description: 'Current status of the channel', example: 'Closed' }
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
 * Fetches channel information by channel id.
 * @returns the channel information
 */
export const getChannel = async (node: Hopr, channelIdStr: string): Promise<ChannelTopologyInfo> => {
  try {
    const channel = await node.db.get_channel(Hash.deserialize(new TextEncoder().encode(channelIdStr)))

    return formatChannelTopologyInfo(node, channel)
  } catch {
    throw Error(STATUS_CODES.CHANNEL_NOT_FOUND)
  }
}

const GET: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context
    const { channelid } = req.params

    try {
      const channel = await getChannel(node, channelid)
      return res.status(200).send(channel)
    } catch (err) {
      const errString = err instanceof Error ? err.message : err?.toString?.() ?? 'Unknown error'

      switch (errString) {
        case STATUS_CODES.INVALID_CHANNELID:
          return res.status(400).send({ status: STATUS_CODES.INVALID_CHANNELID })
        case STATUS_CODES.CHANNEL_NOT_FOUND:
          return res.status(404).send({ status: STATUS_CODES.CHANNEL_NOT_FOUND })
        default:
          return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: errString })
      }
    }
  }
]

GET.apiDoc = {
  description: 'Returns information about the channel.',
  tags: ['Channels'],
  operationId: 'channelsGetChannel',
  parameters: [
    {
      in: 'path',
      name: 'channelid',
      required: true,
      schema: {
        $ref: '#/components/schemas/ChannelId'
      }
    }
  ],
  responses: {
    '200': {
      description: 'Channel fetched succesfully.',
      content: {
        'application/json': {
          schema: {
            items: {
              $ref: '#/components/schemas/ChannelTopology'
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
      description: 'Channel with that channel id was not found. You can list all channels using /channels/ endpoint.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: {
            status: STATUS_CODES.CHANNEL_NOT_FOUND
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

export default { DELETE, GET }
