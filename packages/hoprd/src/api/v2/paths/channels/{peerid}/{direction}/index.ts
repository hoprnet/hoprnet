import type Hopr from '@hoprnet/hopr-core'
import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../../../utils.js'
import { ChannelInfo, formatIncomingChannel, formatOutgoingChannel } from '../../index.js'
import {
  channel_status_to_string,
  ChannelStatus,
  defer,
  generate_channel_id,
  PublicKey,
  type DeferType,
  Address
} from '@hoprnet/hopr-utils'
import { log } from 'debug'

const closingRequests = new Map<string, DeferType<void>>()

/**
 * Closes a channel with provided peerId.
 * @returns Channel status and receipt.
 */
export async function closeChannel(
  node: Hopr,
  peerIdStr: string,
  direction: ChannelInfo['type']
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
  const counterpartyAddress = PublicKey.from_peerid_str(peerIdStr).to_address()

  const channelId = generate_channel_id(node.getEthereumAddress(), counterpartyAddress)

  let closingRequest = closingRequests.get(channelId.to_hex())
  if (closingRequest == null) {
    closingRequest = defer<void>()
    closingRequests.set(channelId.to_hex(), closingRequest)
  } else {
    await closingRequest.promise
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
    const { peerid, direction } = req.params

    const closingResult = await closeChannel(node, peerid, direction as any)

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
      name: 'peerid',
      required: true,
      schema: {
        format: 'peerId',
        type: 'string',
        description: 'PeerId attached to the channel that we want to close.',
        example: '16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12'
      }
    },
    {
      in: 'path',
      name: 'direction',
      description: 'Specify which channel should be fetched, incoming or outgoing.',
      required: true,
      schema: {
        type: 'string',
        enum: ['incoming', 'outgoing'] as ChannelInfo['type'][]
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
      description: 'Invalid peerId.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: {
            status: STATUS_CODES.INVALID_PEERID
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
 * Fetches channel between node and counterparty in the direction provided.
 * @returns the channel between node and counterparty
 */
export const getChannel = async (
  node: Hopr,
  counterparty: string,
  direction: ChannelInfo['type']
): Promise<ChannelInfo> => {
  let counterpartyAddress: Address
  try {
    counterpartyAddress = PublicKey.from_peerid_str(counterparty).to_address()
  } catch (err) {
    throw Error(STATUS_CODES.INVALID_PEERID)
  }

  try {
    return direction === 'outgoing'
      ? await node.getChannel(node.getEthereumAddress(), counterpartyAddress).then(formatOutgoingChannel)
      : await node.getChannel(counterpartyAddress, node.getEthereumAddress()).then(formatIncomingChannel)
  } catch {
    throw Error(STATUS_CODES.CHANNEL_NOT_FOUND)
  }
}

const GET: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context
    const { peerid, direction } = req.params

    if (!['incoming', 'outgoing'].includes(direction)) {
      return res
        .status(404)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: 'Method not supported. Use "incoming" or "outgoing"' })
    }

    try {
      const channel = await getChannel(node, peerid, direction as any)
      return res.status(200).send(channel)
    } catch (err) {
      const errString = err instanceof Error ? err.message : err?.toString?.() ?? 'Unknown error'

      switch (errString) {
        case STATUS_CODES.INVALID_PEERID:
          return res.status(400).send({ status: STATUS_CODES.INVALID_PEERID })
        case STATUS_CODES.CHANNEL_NOT_FOUND:
          return res.status(404).send({ status: STATUS_CODES.CHANNEL_NOT_FOUND })
        default:
          return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: errString })
      }
    }
  }
]

GET.apiDoc = {
  description: 'Returns information about the channel between this node and provided peerId.',
  tags: ['Channels'],
  operationId: 'channelsGetChannel',
  parameters: [
    {
      in: 'path',
      name: 'peerid',
      description: 'Counterparty peerId assigned to the channel you want to fetch.',
      required: true,
      schema: {
        $ref: '#/components/schemas/HoprAddress'
      }
    },
    {
      in: 'path',
      name: 'direction',
      description: 'Specify which channel should be fetched, incoming or outgoing.',
      required: true,
      schema: {
        type: 'string',
        enum: ['incoming', 'outgoing'] as ChannelInfo['type'][]
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
              $ref: '#/components/schemas/Channel'
            }
          }
        }
      }
    },
    '400': {
      description: 'Invalid peerId.',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/RequestStatus'
          },
          example: {
            status: STATUS_CODES.INVALID_PEERID
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
      description: 'Channel with that peerId was not found. You can list all channels using /channels/ endpoint.',
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
