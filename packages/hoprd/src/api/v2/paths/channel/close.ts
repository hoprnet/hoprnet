import type Hopr from '@hoprnet/hopr-core'
import type { Operation } from 'express-openapi'
import PeerId from 'peer-id'
import { STATUS_CODES } from '../../'

export const closeChannel = async (node: Hopr, peerIdStr: string) => {
  let peerId: PeerId
  try {
    peerId = PeerId.createFromB58String(peerIdStr)
  } catch (err) {
    throw Error(STATUS_CODES.INVALID_PEERID)
  }

  const { status: channelStatus, receipt } = await node.closeChannel(peerId)

  return {
    channelStatus,
    receipt
  }
}

export const POST: Operation = [
  async (req, res, _next) => {
    const { node } = req.context
    const { peerId } = req.body

    if (!peerId) {
      return res.status(400).send({ status: 'missingPeerId' })
    }

    try {
      const { receipt, channelStatus } = await closeChannel(node, peerId)
      return res.status(200).send({ status: STATUS_CODES.SUCCESS, receipt, channelStatus })
    } catch (err) {
      if (err.message.includes(STATUS_CODES.INVALID_PEERID)) {
        return res.status(400).send({ status: STATUS_CODES.INVALID_PEERID, error: err.message })
      } else {
        return res.status(500).send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err.message })
      }
    }
  }
]

POST.apiDoc = {
  description: 'Close a channel.',
  tags: ['channel'],
  operationId: 'postChannelClose',
  requestBody: {
    content: {
      'application/json': {
        schema: {
          type: 'object',
          properties: {
            peerId: { type: 'string', description: 'PeerId attached to the channel that we want to close.' }
          },
          example: {
            peerId: '16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12'
          }
        }
      }
    }
  },
  responses: {
    '200': {
      description: 'Channel closed succesfully',
      content: {
        'application/json': {
          schema: {
            type: 'object',
            properties: {
              status: { type: 'string', example: 'success' },
              closureStatus: {
                $ref: '#/components/schemas/ChannelClosureStatus'
              }
            }
          }
        }
      }
    },
    '400': {
      description: 'Invalid peerId',
      content: {
        'application/json': {
          schema: {
            $ref: '#/components/schemas/StatusResponse'
          },
          example: {
            status: 'invalidPeerId | missingPeerId'
          }
        }
      }
    }
  }
}
