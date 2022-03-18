import type Hopr from '@hoprnet/hopr-core'
import type { Operation } from 'express-openapi'
import PeerId from 'peer-id'
import { STATUS_CODES } from '../../../utils'
import { ChannelInfo, channelStatusToString, formatIncomingChannel, formatOutgoingChannel } from '..'

/**
 * Closes a channel with provided peerId.
 * @returns Channel status and receipt.
 */
export const closeChannel = async (node: Hopr, peerIdStr: string, direction: ChannelInfo['type']) => {
  let peerId: PeerId
  try {
    peerId = PeerId.createFromB58String(peerIdStr)
  } catch (err) {
    throw Error(STATUS_CODES.INVALID_PEERID)
  }

  const { status: channelStatus, receipt } = await node.closeChannel(peerId, direction)

  return {
    channelStatus,
    receipt
  }
}

export const DELETE: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { peerid, direction } = req.params

    try {
      const { receipt, channelStatus } = await closeChannel(node, peerid, direction as any)
      return res.status(200).send({ receipt, channelStatus: channelStatusToString(channelStatus) })
    } catch (err) {
      if (err.message.includes(STATUS_CODES.INVALID_PEERID)) {
        return res.status(400).send({ status: STATUS_CODES.INVALID_PEERID })
      } else {
        return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
      }
    }
  }
]

DELETE.apiDoc = {
  description: `Close a opened channel between this node and other node. Once you’ve initiated channel closure, you have to wait for a specified closure time, it will show you a closure initiation message with cool-off time you need to wait.
  Then you will need to send the same command again to finalize closure. This is a cool down period to give the other party in the channel sufficient time to redeem their tickets.`,
  tags: ['Channels'],
  operationId: 'channelsCloseChannel',
  parameters: [
    {
      in: 'path',
      name: 'peerid',
      required: true,
      schema: {
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
              channelStatus: { type: 'number', description: 'Current status of the channel', example: 2 }
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
  let counterpartyPeerId: PeerId
  try {
    counterpartyPeerId = PeerId.createFromB58String(counterparty)
  } catch (err) {
    throw Error(STATUS_CODES.INVALID_PEERID)
  }

  const selfPeerId = node.getId()

  try {
    return direction === 'outgoing'
      ? await node.getChannel(selfPeerId, counterpartyPeerId).then(formatOutgoingChannel)
      : await node.getChannel(counterpartyPeerId, selfPeerId).then(formatIncomingChannel)
  } catch {
    throw Error(STATUS_CODES.CHANNEL_NOT_FOUND)
  }
}

export const GET: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { peerid, direction } = req.params

    try {
      const channel = await getChannel(node, peerid, direction as any)
      return res.status(200).send(channel)
    } catch (err) {
      console.log(err)
      if (err.message === STATUS_CODES.INVALID_PEERID) {
        return res.status(400).send({ status: STATUS_CODES.INVALID_PEERID })
      } else if (err.message === STATUS_CODES.CHANNEL_NOT_FOUND) {
        return res.status(404).send({ status: STATUS_CODES.CHANNEL_NOT_FOUND })
      } else {
        return res.status(422).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
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
